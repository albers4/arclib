// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use ir::Module;

use crate::{
    CompileRequest, ConstraintPredicate, ConstraintScope, ConstraintStrength, DecisionSubject,
    Metric, ObjectiveDirection, PipelineRegistry, PolicyDecision, PolicyDecisionKind, PolicyValue,
    Property,
};

use super::{
    ConversionPlan, ConversionRegistry, ConversionStage, ConversionStep, PlanningConfig,
    PlanningError,
};

struct ScoredRoute {
    indices: Vec<usize>,

    score: f64,

    properties: HashMap<Property, PolicyValue>,

    metrics: HashMap<Metric, f64>,

    decisions: Vec<PolicyDecision>,
}

pub fn plan_conversion(
    module: &Module,

    request: &CompileRequest,

    source: impl Into<ConversionStage>,

    target: impl Into<ConversionStage>,

    registry: &ConversionRegistry,

    pipelines: &PipelineRegistry,

    config: &PlanningConfig,
) -> Result<ConversionPlan, PlanningError> {
    let source = source.into();

    let target = target.into();

    if source == target {
        return Ok(ConversionPlan::new(
            source.clone(),
            target,
            Vec::new(),
            0.0,
            HashMap::new(),
            HashMap::new(),
            vec![route_decision(&source, &source, &[])],
        ));
    }

    let mut candidates = Vec::new();

    let mut path = Vec::new();

    let mut visited = HashSet::from([source.clone()]);

    let mut missing_pipeline = None;

    enumerate_routes(
        module,
        request,
        &source,
        &target,
        registry,
        pipelines,
        config,
        &mut visited,
        &mut path,
        &mut candidates,
        &mut missing_pipeline,
    );

    let relevant_properties = registry.properties();

    let mut best: Option<ScoredRoute> = None;

    for candidate in candidates {
        let Some(scored) = score_route(request, &candidate, registry, &relevant_properties) else {
            continue;
        };

        let replace = best.as_ref().map_or(true, |current| {
            scored.score < current.score
                || (scored.score == current.score && scored.indices.len() < current.indices.len())
        });

        if replace {
            best = Some(scored);
        }
    }

    let Some(mut best) = best else {
        if let Some((edge, pipeline)) = missing_pipeline {
            return Err(PlanningError::MissingPipeline { edge, pipeline });
        }

        return Err(PlanningError::NoRoute { source, target });
    };

    let steps: Vec<ConversionStep> = best
        .indices
        .iter()
        .map(|index| {
            let edge = registry.edge(*index).expect(
                "planned edge index \
                             must exist",
            );

            ConversionStep::new(
                edge.name(),
                edge.source().clone(),
                edge.target().clone(),
                edge.pipeline(),
            )
        })
        .collect();

    best.decisions
        .insert(0, route_decision(&source, &target, &steps));

    Ok(ConversionPlan::new(
        source,
        target,
        steps,
        best.score,
        best.properties,
        best.metrics,
        best.decisions,
    ))
}

#[allow(clippy::too_many_arguments)]
fn enumerate_routes(
    module: &Module,

    request: &CompileRequest,

    current: &ConversionStage,

    target: &ConversionStage,

    registry: &ConversionRegistry,

    pipelines: &PipelineRegistry,

    config: &PlanningConfig,

    visited: &mut HashSet<ConversionStage>,

    path: &mut Vec<usize>,

    candidates: &mut Vec<Vec<usize>>,

    missing_pipeline: &mut Option<(String, String)>,
) {
    if candidates.len() >= config.max_candidates || path.len() >= config.max_depth {
        return;
    }

    for edge_index in registry.outgoing_indices(current) {
        let edge = registry.edge(edge_index).expect(
            "registered edge index \
                     must exist",
        );

        if !edge.supports_target(request) {
            continue;
        }

        if !edge.applicability(module, request).is_applicable() {
            continue;
        }

        if pipelines.get(edge.pipeline()).is_none() {
            if missing_pipeline.is_none() {
                *missing_pipeline = Some((edge.name().to_owned(), edge.pipeline().to_owned()));
            }

            continue;
        }

        if visited.contains(edge.target()) {
            continue;
        }

        path.push(edge_index);

        if edge.target() == target {
            candidates.push(path.clone());
        } else {
            visited.insert(edge.target().clone());

            enumerate_routes(
                module,
                request,
                edge.target(),
                target,
                registry,
                pipelines,
                config,
                visited,
                path,
                candidates,
                missing_pipeline,
            );

            visited.remove(edge.target());
        }

        path.pop();

        if candidates.len() >= config.max_candidates {
            return;
        }
    }
}

fn score_route(
    request: &CompileRequest,

    indices: &[usize],

    registry: &ConversionRegistry,

    relevant_properties: &HashSet<Property>,
) -> Option<ScoredRoute> {
    let mut score = 0.0;

    let mut properties = HashMap::new();

    let mut metrics: HashMap<Metric, f64> = HashMap::new();

    for index in indices {
        let edge = registry.edge(*index)?;

        score += edge.base_cost();

        for (property, value) in edge.properties() {
            if let Some(existing) = properties.get(property) {
                if existing != value {
                    return None;
                }
            } else {
                properties.insert(property.clone(), value.clone());
            }
        }

        for (metric, estimate) in edge.metrics() {
            *metrics.entry(metric.clone()).or_insert(0.0) += estimate;
        }
    }

    let mut decisions = Vec::new();

    for constraint in request.constraints().iter() {
        if !matches!(constraint.scope(), ConstraintScope::Global) {
            continue;
        }

        if !relevant_properties.contains(constraint.property()) {
            // This constraint belongs to a later
            // pass or target-specific planner.
            continue;
        }

        let selected = properties.get(constraint.property());

        let satisfied = predicate_matches(constraint.predicate(), selected);

        match constraint.strength() {
            ConstraintStrength::Required => {
                if !satisfied {
                    return None;
                }

                decisions.push(constraint_decision(
                    PolicyDecisionKind::RequiredSatisfied,
                    constraint.property().clone(),
                    selected,
                    true,
                ));
            }

            ConstraintStrength::Preferred { weight } => {
                if !satisfied {
                    score += weight;
                }

                decisions.push(constraint_decision(
                    if satisfied {
                        PolicyDecisionKind::PreferenceSelected
                    } else {
                        PolicyDecisionKind::PreferenceRejected
                    },
                    constraint.property().clone(),
                    selected,
                    satisfied,
                ));
            }

            ConstraintStrength::Hint => {
                if !satisfied {
                    score += 0.1;
                }

                decisions.push(constraint_decision(
                    if satisfied {
                        PolicyDecisionKind::HintApplied
                    } else {
                        PolicyDecisionKind::HintIgnored
                    },
                    constraint.property().clone(),
                    selected,
                    satisfied,
                ));
            }
        }
    }

    for objective in request.objectives().iter() {
        if !matches!(objective.scope(), ConstraintScope::Global) {
            continue;
        }

        let estimate = metrics.get(objective.metric()).copied().unwrap_or(0.0);

        match objective.direction() {
            ObjectiveDirection::Minimize => {
                score += objective.weight() * estimate;
            }

            ObjectiveDirection::Maximize => {
                score -= objective.weight() * estimate;
            }
        }
    }

    Some(ScoredRoute {
        indices: indices.to_vec(),

        score,
        properties,
        metrics,
        decisions,
    })
}

fn predicate_matches(predicate: &ConstraintPredicate, selected: Option<&PolicyValue>) -> bool {
    match predicate {
        ConstraintPredicate::Equals(expected) => selected == Some(expected),

        ConstraintPredicate::NotEquals(forbidden) => {
            selected.is_some_and(|value| value != forbidden)
        }

        ConstraintPredicate::OneOf(candidates) => {
            selected.is_some_and(|value| candidates.contains(value))
        }

        ConstraintPredicate::AtLeast(minimum) => numeric(selected)
            .zip(numeric(Some(minimum)))
            .is_some_and(|(selected, minimum)| selected >= minimum),

        ConstraintPredicate::AtMost(maximum) => numeric(selected)
            .zip(numeric(Some(maximum)))
            .is_some_and(|(selected, maximum)| selected <= maximum),

        ConstraintPredicate::Present => selected.is_some(),
    }
}

fn numeric(value: Option<&PolicyValue>) -> Option<f64> {
    match value? {
        PolicyValue::Integer(value) => Some(*value as f64),

        PolicyValue::Float(value) => Some(*value),

        _ => None,
    }
}

fn constraint_decision(
    kind: PolicyDecisionKind,

    property: Property,

    selected: Option<&PolicyValue>,

    satisfied: bool,
) -> PolicyDecision {
    let mut decision = PolicyDecision::for_property(
        kind,
        ConstraintScope::Global,
        property,
        if satisfied {
            "selected conversion route \
                 satisfies the planning \
                 constraint"
        } else {
            "selected conversion route \
                 does not satisfy the \
                 preference or hint"
        },
    );

    if let Some(value) = selected {
        decision = decision.selected(value.clone());
    }

    decision
}

fn route_decision(
    source: &ConversionStage,
    target: &ConversionStage,
    steps: &[ConversionStep],
) -> PolicyDecision {
    let route = if steps.is_empty() {
        source.to_string()
    } else {
        let mut route = source.to_string();

        for step in steps {
            route.push_str(" -> ");
            route.push_str(step.target().as_str());
        }

        route
    };

    PolicyDecision::new(
        PolicyDecisionKind::Other(Arc::from("conversion-route")),
        ConstraintScope::Global,
        DecisionSubject::named("conversion.route"),
        format!(
            "selected a {}-step conversion route \
             from '{source}' to '{target}'",
            steps.len(),
        ),
    )
    .selected(route)
}

#[cfg(test)]
mod tests {
    use ir::Module;

    use crate::{
        CompileRequest, ConversionEdgeDescriptor, ConversionRegistry, PipelineDescriptor,
        PipelineRegistry,
    };

    use super::*;

    #[test]
    fn selects_lower_cost_route() {
        let mut conversions = ConversionRegistry::new();

        conversions
            .register(
                ConversionEdgeDescriptor::new("direct", "a", "c", "direct").with_base_cost(10.0),
            )
            .unwrap();

        conversions
            .register(
                ConversionEdgeDescriptor::new("a-to-b", "a", "b", "a-to-b").with_base_cost(1.0),
            )
            .unwrap();

        conversions
            .register(
                ConversionEdgeDescriptor::new("b-to-c", "b", "c", "b-to-c").with_base_cost(1.0),
            )
            .unwrap();

        let mut pipelines = PipelineRegistry::new();

        for name in ["direct", "a-to-b", "b-to-c"] {
            pipelines.register(PipelineDescriptor::new(name)).unwrap();
        }

        let module = Module::new();

        let request = CompileRequest::default();

        let plan = plan_conversion(
            &module,
            &request,
            "a",
            "c",
            &conversions,
            &pipelines,
            &PlanningConfig::default(),
        )
        .unwrap();

        assert_eq!(plan.steps().len(), 2,);

        assert_eq!(plan.steps()[0].edge(), "a-to-b",);

        assert_eq!(plan.steps()[1].edge(), "b-to-c",);
    }

    #[test]
    fn required_property_selects_method() {
        let mut conversions = ConversionRegistry::new();

        conversions
            .register(
                ConversionEdgeDescriptor::new("through-lbm", "operator", "target", "through-lbm")
                    .property("method.name", "lbm"),
            )
            .unwrap();

        conversions
            .register(
                ConversionEdgeDescriptor::new("through-fvm", "operator", "target", "through-fvm")
                    .property("method.name", "fvm"),
            )
            .unwrap();

        let mut pipelines = PipelineRegistry::new();

        pipelines
            .register(PipelineDescriptor::new("through-lbm"))
            .unwrap();

        pipelines
            .register(PipelineDescriptor::new("through-fvm"))
            .unwrap();

        let request = CompileRequest::builder("unused")
            .require("method.name", "fvm")
            .build()
            .unwrap();

        let plan = plan_conversion(
            &Module::new(),
            &request,
            "operator",
            "target",
            &conversions,
            &pipelines,
            &PlanningConfig::default(),
        )
        .unwrap();

        assert_eq!(plan.steps().len(), 1,);

        assert_eq!(plan.steps()[0].edge(), "through-fvm",);
    }
}
