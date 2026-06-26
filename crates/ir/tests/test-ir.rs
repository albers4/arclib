use ir::{
    Attribute, BlockBuilder, DialectDescriptor, DialectRegistry, DialectRegistryError,
    GreedyRewriteConfig, InsertionPoint, IrError, Module, OperationBuilder, OperationDescriptor,
    OperationId, OperationRef, PatternBenefit, PatternResult, PatternRewriter, Pure, RewriteError,
    RewritePattern, RewritePatternSet, SymbolError, SymbolRef, SymbolVisibility, Type,
    UnknownOperationPolicy, ValueProducer, apply_patterns_greedily,
};

#[test]
fn creates_valid_module() {
    let module = Module::named("hello");

    assert_eq!(module.name(), Some("hello"));
    assert!(module.is_empty());
    assert!(module.verify_structure().is_ok());
}

#[test]
fn stores_module_attributes() {
    let mut module = Module::new();

    module.set_attribute("target", Attribute::string("cuda"));

    assert_eq!(
        module.attribute("target").and_then(Attribute::as_str),
        Some("cuda")
    )
}

#[test]
fn inserts_operation_and_results() {
    let mut module = Module::named("simulation");

    let constant = module
        .append_operation(
            OperationBuilder::new("arith.constant")
                .result(Type::f64())
                .attribute("value", Attribute::Float(1.0)),
            [],
        )
        .unwrap();

    let operation = module.operation(constant).unwrap();

    assert_eq!(operation.name().as_str(), "arith.constant",);

    let result = operation.result(0).unwrap();

    assert_eq!(module.value(result).unwrap().ty(), &Type::f64(),);

    module.verify().unwrap();
}

#[test]
fn maintains_use_def_information() {
    let mut module = Module::new();

    let lhs = module
        .append_operation(
            OperationBuilder::new("arith.constant").result(Type::f64()),
            [],
        )
        .unwrap();

    let rhs = module
        .append_operation(
            OperationBuilder::new("arith.constant").result(Type::f64()),
            [],
        )
        .unwrap();

    let lhs_value = module.operation(lhs).unwrap().result(0).unwrap();

    let rhs_value = module.operation(rhs).unwrap().result(0).unwrap();

    let add = module
        .append_operation(
            OperationBuilder::new("arith.add").result(Type::f64()),
            [lhs_value, rhs_value],
        )
        .unwrap();

    assert_eq!(module.value(lhs_value).unwrap().uses()[0].user, add);

    assert_eq!(module.value(rhs_value).unwrap().uses()[0].user, add);

    module.verify().unwrap();
}

#[test]
fn rejects_values_from_another_module() {
    let mut first = Module::new();
    let mut second = Module::new();

    let producer = first
        .append_operation(
            OperationBuilder::new("test.producer").result(Type::f64()),
            [],
        )
        .unwrap();

    let foreign_value = first.operation(producer).unwrap().result(0).unwrap();

    let error = second
        .append_operation(OperationBuilder::new("test.consumer"), [foreign_value])
        .unwrap_err();

    assert!(matches!(error, IrError::ForeignHandle { kind: "value" }));
}

#[test]
fn creates_nested_regions_on_insertion() {
    let mut module = Module::new();

    let operation = module
        .append_operation(OperationBuilder::new("control.region").regions(2), [])
        .unwrap();

    assert_eq!(module.operation(operation).unwrap().regions().len(), 2,);

    module.verify().unwrap();
}

#[test]
fn creates_block_arguments() {
    let mut module = Module::new();

    let container = module
        .append_operation(OperationBuilder::new("control.container").region(), [])
        .unwrap();

    let region = module.operation(container).unwrap().regions()[0];

    let block = module
        .append_block(
            region,
            BlockBuilder::new()
                .argument(Type::f64())
                .argument(Type::f64()),
        )
        .unwrap();

    let lhs = module.block(block).unwrap().arguments()[0];

    let rhs = module.block(block).unwrap().arguments()[1];

    let add = module
        .append_operation_to_block(
            block,
            OperationBuilder::new("arith.add").result(Type::f64()),
            [lhs, rhs],
        )
        .unwrap();

    assert_eq!(module.value(lhs).unwrap().uses()[0].user, add,);

    assert_eq!(module.value(rhs).unwrap().uses()[0].user, add,);

    module.verify().unwrap();
}

#[test]
fn inserts_operations_at_specific_points() {
    let mut module = Module::new();

    let first = module
        .append_operation(OperationBuilder::new("test.first"), [])
        .unwrap();

    let third = module
        .append_operation(OperationBuilder::new("test.third"), [])
        .unwrap();

    let second = module
        .insert_operation(
            InsertionPoint::Before(third),
            OperationBuilder::new("test.second"),
            [],
        )
        .unwrap();

    let operations = module
        .block(module.body_block())
        .unwrap()
        .operations()
        .to_vec();

    assert_eq!(operations, vec![first, second, third],);

    module.verify().unwrap();
}

#[test]
fn allows_multiple_blocks_in_module_body() {
    let mut module = Module::new();

    let second_block = module
        .append_block(module.body_region(), BlockBuilder::new())
        .unwrap();

    module
        .append_operation_to_block(second_block, OperationBuilder::new("test.operation"), [])
        .unwrap();

    assert_eq!(
        module.region(module.body_region()).unwrap().blocks().len(),
        2,
    );

    module.verify().unwrap();
}

#[test]
fn module_is_a_symbol_table() {
    let module = Module::new();

    assert!(
        module
            .operation(module.root_operation())
            .unwrap()
            .is_symbol_table()
    );

    module.verify().unwrap();
}

#[test]
fn defines_and_looks_up_symbol() {
    let mut module = Module::new();

    let domain = module
        .append_operation(
            OperationBuilder::new("geom.domain")
                .symbol("cavity")
                .visibility(SymbolVisibility::Public)
                .region(),
            [],
        )
        .unwrap();

    let found = module
        .lookup_symbol(module.root_operation(), "cavity")
        .unwrap();

    assert_eq!(found, Some(domain));
    module.verify().unwrap();
}

#[test]
fn rejects_duplicate_symbols() {
    let mut module = Module::new();

    module
        .append_operation(OperationBuilder::new("geom.domain").symbol("cavity"), [])
        .unwrap();

    module
        .append_operation(OperationBuilder::new("geom.domain").symbol("cavity"), [])
        .unwrap();

    let error = module.verify_symbols().unwrap_err();

    assert!(matches!(error, SymbolError::DuplicateSymbol { .. }));
}

#[test]
fn resolves_absolute_nested_symbol() {
    let mut module = Module::new();

    let namespace = module
        .append_operation(
            OperationBuilder::new("builtin.namespace")
                .symbol("physics")
                .symbol_table()
                .region(),
            [],
        )
        .unwrap();

    let region = module.operation(namespace).unwrap().regions()[0];

    let block = module.append_block(region, BlockBuilder::new()).unwrap();

    let fluid = module
        .append_operation_to_block(
            block,
            OperationBuilder::new("fluid.system").symbol("fluid"),
            [],
        )
        .unwrap();

    let user = module
        .append_operation(
            OperationBuilder::new("test.use").attribute(
                "target",
                Attribute::symbol_ref(SymbolRef::absolute("physics").nested("fluid")),
            ),
            [],
        )
        .unwrap();

    let resolved = module
        .resolve_symbol(user, &SymbolRef::absolute("physics").nested("fluid"))
        .unwrap();

    assert_eq!(resolved, fluid);
    module.verify().unwrap();
}

#[test]
fn resolves_relative_symbol_in_nearest_scope() {
    let mut module = Module::new();

    let namespace = module
        .append_operation(
            OperationBuilder::new("builtin.namespace")
                .symbol("physics")
                .symbol_table()
                .region(),
            [],
        )
        .unwrap();

    let region = module.operation(namespace).unwrap().regions()[0];

    let block = module.append_block(region, BlockBuilder::new()).unwrap();

    let target = module
        .append_operation_to_block(
            block,
            OperationBuilder::new("fluid.system").symbol("fluid"),
            [],
        )
        .unwrap();

    let user = module
        .append_operation_to_block(
            block,
            OperationBuilder::new("test.use").attribute(
                "target",
                Attribute::symbol_ref(SymbolRef::relative("fluid")),
            ),
            [],
        )
        .unwrap();

    let resolved = module
        .resolve_symbol(user, &SymbolRef::relative("fluid"))
        .unwrap();

    assert_eq!(resolved, target);
    module.verify().unwrap();
}

#[test]
fn rejects_unresolved_symbol_reference() {
    let mut module = Module::new();

    module
        .append_operation(
            OperationBuilder::new("test.use").attribute(
                "target",
                Attribute::symbol_ref(SymbolRef::relative("missing")),
            ),
            [],
        )
        .unwrap();

    let error = module.verify_symbols().unwrap_err();

    assert!(matches!(error, SymbolError::UnresolvedReference { .. }));
}

fn create_arithmetic_registry() -> DialectRegistry {
    let mut registry = DialectRegistry::with_builtin();

    let mut arithmetic = DialectDescriptor::new("arith");

    arithmetic
        .register_operation(
            OperationDescriptor::new("arith.constant")
                .with_trait::<Pure>()
                .with_verifier(|operation: OperationRef| {
                    if !operation.operands().is_empty() {
                        return Err("constant must not have operands".into());
                    }

                    if operation.results().len() != 1 {
                        return Err("constant must have one result".into());
                    }

                    Ok(())
                }),
        )
        .unwrap();

    arithmetic
        .register_operation(
            OperationDescriptor::new("arith.add")
                .with_trait::<Pure>()
                .with_verifier(|operation: OperationRef| {
                    if operation.operands().len() != 2 {
                        return Err("add must have two operands".into());
                    }

                    if operation.results().len() != 1 {
                        return Err("add must have one result".into());
                    }

                    Ok(())
                }),
        )
        .unwrap();

    registry.register_dialect(arithmetic).unwrap();

    registry
}

#[test]
fn verifies_registered_operations() {
    let registry = create_arithmetic_registry();

    let mut module = Module::new();

    let lhs = module
        .append_operation(
            OperationBuilder::new("arith.constant").result(Type::f64()),
            [],
        )
        .unwrap();

    let rhs = module
        .append_operation(
            OperationBuilder::new("arith.constant").result(Type::f64()),
            [],
        )
        .unwrap();

    let lhs_value = module.operation(lhs).unwrap().result(0).unwrap();

    let rhs_value = module.operation(rhs).unwrap().result(0).unwrap();

    module
        .append_operation(
            OperationBuilder::new("arith.add").result(Type::f64()),
            [lhs_value, rhs_value],
        )
        .unwrap();

    module
        .verify_with_registry(&registry, UnknownOperationPolicy::Reject)
        .unwrap();
}

#[test]
fn reports_operation_verifier_failure() {
    let registry = create_arithmetic_registry();

    let mut module = Module::new();

    module
        .append_operation(OperationBuilder::new("arith.add").result(Type::f64()), [])
        .unwrap();

    let error = module
        .verify_with_registry(&registry, UnknownOperationPolicy::Reject)
        .unwrap_err();

    assert!(matches!(
        error,
        crate::IrError::Dialect(DialectRegistryError::OperationVerificationFailed { .. })
    ));
}

#[test]
fn rejects_unknown_operation() {
    let registry = DialectRegistry::with_builtin();

    let mut module = Module::new();

    module
        .append_operation(OperationBuilder::new("fluid.system"), [])
        .unwrap();

    let error = module
        .verify_with_registry(&registry, UnknownOperationPolicy::Reject)
        .unwrap_err();

    assert!(matches!(
        error,
        crate::IrError::Dialect(DialectRegistryError::UnknownDialect { .. })
    ));
}

#[test]
fn can_allow_unknown_operations() {
    let registry = DialectRegistry::with_builtin();

    let mut module = Module::new();

    module
        .append_operation(OperationBuilder::new("fluid.system"), [])
        .unwrap();

    module
        .verify_with_registry(&registry, UnknownOperationPolicy::Allow)
        .unwrap();
}

#[test]
fn supports_typed_interfaces() {
    struct EstimatedCost {
        operations: usize,
    }

    let descriptor =
        OperationDescriptor::new("test.operation").with_interface(EstimatedCost { operations: 12 });

    let interface = descriptor.interface::<EstimatedCost>().unwrap();

    assert_eq!(interface.operations, 12,);
}

#[test]
fn supports_operation_traits() {
    let descriptor = OperationDescriptor::new("arith.constant").with_trait::<Pure>();

    assert!(descriptor.has_trait::<Pure>());
}

#[test]
fn rejects_mismatched_operation_dialect() {
    let mut fluid = DialectDescriptor::new("fluid");

    let error = fluid
        .register_operation(OperationDescriptor::new("solid.system"))
        .unwrap_err();

    assert!(matches!(
        error,
        DialectRegistryError::OperationDialectMismatch { .. }
    ));
}

struct AddZeroPattern;

impl RewritePattern for AddZeroPattern {
    fn name(&self) -> &'static str {
        "AddZeroPattern"
    }

    fn match_and_rewrite(
        &self,
        operation: OperationId,
        rewriter: &mut PatternRewriter<'_>,
    ) -> Result<PatternResult, RewriteError> {
        let (lhs, rhs) = {
            let operation = rewriter.operation(operation).unwrap();

            if operation.operands().len() != 2 || operation.results().len() != 1 {
                return Ok(PatternResult::NoMatch);
            }

            (operation.operands()[0], operation.operands()[1])
        };

        let rhs_producer = rewriter.value(rhs).unwrap().producer();

        let ValueProducer::OperationResult {
            operation: rhs_operation,
            ..
        } = rhs_producer
        else {
            return Ok(PatternResult::NoMatch);
        };

        let is_zero = matches!(
            rewriter
                .operation(rhs_operation)
                .unwrap()
                .attribute("value"),
            Some(Attribute::Float(value))
                if *value == 0.0
        );

        if !is_zero {
            return Ok(PatternResult::NoMatch);
        }

        rewriter.replace_operation(operation, &[lhs])?;

        Ok(PatternResult::Rewritten)
    }
}

#[test]
fn replaces_operation_results_and_uses() {
    let mut module = Module::new();

    let lhs_operation = module
        .append_operation(
            OperationBuilder::new("arith.constant")
                .result(Type::f64())
                .attribute("value", Attribute::Float(4.0)),
            [],
        )
        .unwrap();

    let zero_operation = module
        .append_operation(
            OperationBuilder::new("arith.constant")
                .result(Type::f64())
                .attribute("value", Attribute::Float(0.0)),
            [],
        )
        .unwrap();

    let lhs = module.operation(lhs_operation).unwrap().result(0).unwrap();

    let zero = module.operation(zero_operation).unwrap().result(0).unwrap();

    let add = module
        .append_operation(
            OperationBuilder::new("arith.add").result(Type::f64()),
            [lhs, zero],
        )
        .unwrap();

    let add_result = module.operation(add).unwrap().result(0).unwrap();

    let consumer = module
        .append_operation(OperationBuilder::new("test.consume"), [add_result])
        .unwrap();

    let mut patterns = RewritePatternSet::new();

    patterns.add("arith.add", PatternBenefit::DEFAULT, AddZeroPattern);

    apply_patterns_greedily(&mut module, &patterns, &GreedyRewriteConfig::default()).unwrap();

    assert!(module.operation(add).is_none());

    assert_eq!(module.operation(consumer).unwrap().operand(0), Some(lhs),);

    module.verify().unwrap();
}
