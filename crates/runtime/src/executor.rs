// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{ffi::c_void, ptr};

use execution::{BufferCopy, BufferFill, ExecutionCommand, ExecutionPlan, MemorySpace};

use crate::{
    AllocatorRegistry, LinkedKernelRuntime, ResourceStore, RuntimeError, materialize_execution_plan,
};

pub struct LocalExecutor {
    kernels: LinkedKernelRuntime,
    allocators: AllocatorRegistry,
    cuda_device: Option<u32>,
}

impl Default for LocalExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalExecutor {
    pub fn new() -> Self {
        Self {
            kernels: LinkedKernelRuntime::new(),
            allocators: AllocatorRegistry::with_host_allocator(),
            cuda_device: None,
        }
    }

    pub fn kernels(&self) -> &LinkedKernelRuntime {
        &self.kernels
    }

    pub fn kernels_mut(&mut self) -> &mut LinkedKernelRuntime {
        &mut self.kernels
    }

    pub fn allocators(&self) -> &AllocatorRegistry {
        &self.allocators
    }

    pub fn allocators_mut(&mut self) -> &mut AllocatorRegistry {
        &mut self.allocators
    }

    pub fn set_cuda_device(&mut self, ordinal: u32) {
        self.cuda_device = Some(ordinal);
    }

    pub fn clear_cuda_device(&mut self) {
        self.cuda_device = None;
    }

    pub fn prepare(&self, plan: &ExecutionPlan) -> Result<ResourceStore, RuntimeError> {
        let mut resources = ResourceStore::from_declarations(plan.resources().clone());
        materialize_execution_plan(plan, &mut resources, &self.allocators)?;
        Ok(resources)
    }

    pub fn execute(
        &self,
        plan: &ExecutionPlan,
        resources: &mut ResourceStore,
    ) -> Result<(), RuntimeError> {
        if resources.declarations() != plan.resources() {
            return Err(RuntimeError::ResourceTableMismatch);
        }

        for node_id in plan.schedule().order() {
            let command = plan
                .graph()
                .command(*node_id)
                .expect("execution plan schedule contains only existing nodes");

            match command {
                ExecutionCommand::Kernel(invocation) => {
                    let parameters = invocation.descriptor().abi().parameters();
                    let mut packed = Vec::<*mut c_void>::with_capacity(parameters.len());
                    for (parameter, resource) in parameters
                        .iter()
                        .zip(invocation.arguments().iter().copied())
                    {
                        packed.push(resources.packed_pointer(
                            resource,
                            parameter.kind(),
                            invocation.descriptor().backend(),
                            self.cuda_device,
                        )?);
                    }

                    unsafe {
                        self.kernels
                            .invoke(invocation.descriptor().name(), &mut packed)?;
                    }
                }
                ExecutionCommand::Copy(copy) => execute_copy(*copy, resources)?,
                ExecutionCommand::Fill(fill) => execute_fill(*fill, resources)?,
                ExecutionCommand::Barrier => {
                    // The local executor is sequential. Reaching this node already
                    // establishes the barrier's ordering semantics.
                }
            }
        }

        Ok(())
    }

    pub fn run(&self, plan: &ExecutionPlan) -> Result<ResourceStore, RuntimeError> {
        let mut resources = self.prepare(plan)?;
        self.execute(plan, &mut resources)?;
        Ok(resources)
    }
}

fn execute_copy(copy: BufferCopy, resources: &ResourceStore) -> Result<(), RuntimeError> {
    let source = resources.buffer(copy.source())?;
    let destination = resources.buffer(copy.destination())?;
    validate_host_operation("buffer copy", source.memory_space())?;
    validate_host_operation("buffer copy", destination.memory_space())?;

    if source.bytes() < copy.bytes() {
        return Err(RuntimeError::BufferTooSmall {
            resource: copy.source(),
            required: copy.bytes(),
            actual: source.bytes(),
        });
    }
    if destination.bytes() < copy.bytes() {
        return Err(RuntimeError::BufferTooSmall {
            resource: copy.destination(),
            required: copy.bytes(),
            actual: destination.bytes(),
        });
    }

    unsafe {
        ptr::copy(
            source.pointer().as_ptr().cast::<u8>(),
            destination.pointer().as_ptr().cast::<u8>(),
            copy.bytes(),
        );
    }
    Ok(())
}

fn execute_fill(fill: BufferFill, resources: &ResourceStore) -> Result<(), RuntimeError> {
    let destination = resources.buffer(fill.destination())?;
    validate_host_operation("buffer fill", destination.memory_space())?;
    if destination.bytes() < fill.bytes() {
        return Err(RuntimeError::BufferTooSmall {
            resource: fill.destination(),
            required: fill.bytes(),
            actual: destination.bytes(),
        });
    }

    unsafe {
        ptr::write_bytes(
            destination.pointer().as_ptr().cast::<u8>(),
            fill.value(),
            fill.bytes(),
        );
    }
    Ok(())
}

fn validate_host_operation(
    operation: &'static str,
    memory_space: MemorySpace,
) -> Result<(), RuntimeError> {
    if matches!(memory_space, MemorySpace::Host | MemorySpace::Unified) {
        Ok(())
    } else {
        Err(RuntimeError::UnsupportedBufferOperation {
            operation,
            memory_space,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_void, mem::size_of, sync::Arc};

    use execution::{
        BufferCopy, BufferFill, BufferSpec, ExecutionGraphBuilder, ExecutionPlan, KernelInvocation,
        MemorySpace, ResourceTable, ScalarValue,
    };
    use kernel::{
        KernelAbi, KernelAccess, KernelBackend, KernelDescriptor, KernelParameter, KernelValueKind,
    };

    use crate::LocalExecutor;

    unsafe extern "C" fn increment_f64(
        arguments: *const *mut c_void,
        argument_count: usize,
    ) -> i32 {
        if argument_count != 1 {
            return 1;
        }
        let pointer = unsafe { *arguments }.cast::<f64>();
        unsafe { *pointer += 1.0 };
        0
    }

    #[test]
    fn executes_kernel_command() {
        let descriptor = Arc::new(
            KernelDescriptor::new("test.increment", "increment_f64", KernelBackend::Cpu).with_abi(
                KernelAbi::new()
                    .parameter(KernelParameter::new(
                        "value",
                        KernelValueKind::Scalar,
                        KernelAccess::ReadWrite,
                    ))
                    .result_alias(0),
            ),
        );

        let mut resources = ResourceTable::new();
        let value = resources.declare_scalar(ScalarValue::F64(41.0));
        let mut graph = ExecutionGraphBuilder::new();
        graph
            .add_kernel(KernelInvocation::new(descriptor, [value]).unwrap())
            .unwrap();
        let plan = ExecutionPlan::new(resources, graph.finish()).unwrap();

        let mut executor = LocalExecutor::new();
        executor
            .kernels_mut()
            .register("test.increment", increment_f64)
            .unwrap();

        let store = executor.run(&plan).unwrap();
        assert_eq!(store.scalar(value).unwrap().as_f64(), Some(42.0));
    }

    #[test]
    fn executes_fill_and_copy_commands() {
        let mut resources = ResourceTable::new();
        let source = resources.declare_buffer(BufferSpec::new(8, MemorySpace::Host));
        let destination = resources.declare_buffer(BufferSpec::new(8, MemorySpace::Host));

        let mut graph = ExecutionGraphBuilder::new();
        graph
            .add_fill(BufferFill::new(source, 0x2a, 8).unwrap())
            .unwrap();
        graph
            .add_copy(BufferCopy::new(source, destination, 8).unwrap())
            .unwrap();
        let plan = ExecutionPlan::new(resources, graph.finish()).unwrap();

        let executor = LocalExecutor::new();
        let store = executor.run(&plan).unwrap();
        let output = store.buffer(destination).unwrap();
        let bytes =
            unsafe { std::slice::from_raw_parts(output.pointer().as_ptr().cast::<u8>(), 8) };
        assert_eq!(bytes, &[0x2a; 8]);
        assert!(output.bytes() >= size_of::<u64>());
    }
}
