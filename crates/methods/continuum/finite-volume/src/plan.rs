// Copyright (c) 2026 ARC (Applied Research & Computation)
// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{mem::size_of, sync::Arc};

use execution::{
    BufferCopy, BufferSpec, ExecutionGraphBuilder, ExecutionPlan, KernelInvocation, MemorySpace,
    ResourceId, ResourceTable, ScalarValue,
};
use mesh::StructuredLineMesh;

use crate::{FvmError, FvmKernelSet, FvmLineMeshView};

#[derive(Debug, Clone, Copy)]
pub struct DiffusionResources {
    state: ResourceId,
    rhs: ResourceId,
    scaled_rhs: ResourceId,
    next_state: ResourceId,
    owner: ResourceId,
    neighbour: ResourceId,
    face_coefficients: ResourceId,
    cell_volumes: ResourceId,
    diffusivity: ResourceId,
    time_step: ResourceId,
    cell_count: ResourceId,
    face_count: ResourceId,
}

impl DiffusionResources {
    pub const fn state(self) -> ResourceId {
        self.state
    }
    pub const fn rhs(self) -> ResourceId {
        self.rhs
    }
    pub const fn scaled_rhs(self) -> ResourceId {
        self.scaled_rhs
    }
    pub const fn next_state(self) -> ResourceId {
        self.next_state
    }
    pub const fn owner(self) -> ResourceId {
        self.owner
    }
    pub const fn neighbour(self) -> ResourceId {
        self.neighbour
    }
    pub const fn face_coefficients(self) -> ResourceId {
        self.face_coefficients
    }
    pub const fn cell_volumes(self) -> ResourceId {
        self.cell_volumes
    }
    pub const fn diffusivity(self) -> ResourceId {
        self.diffusivity
    }
    pub const fn time_step(self) -> ResourceId {
        self.time_step
    }
    pub const fn cell_count(self) -> ResourceId {
        self.cell_count
    }
    pub const fn face_count(self) -> ResourceId {
        self.face_count
    }
}

#[derive(Debug, Clone)]
pub struct Diffusion1dPlan {
    plan: ExecutionPlan,
    resources: DiffusionResources,
    view: FvmLineMeshView,
    initial_state: Vec<f64>,
}

impl Diffusion1dPlan {
    pub fn new(
        mesh: &StructuredLineMesh,
        initial_state: Vec<f64>,
        diffusivity: f64,
        time_step: f64,
    ) -> Result<Self, FvmError> {
        if initial_state.len() != mesh.cells() {
            return Err(FvmError::InitialStateLength {
                expected: mesh.cells(),
                actual: initial_state.len(),
            });
        }
        if !diffusivity.is_finite() || diffusivity < 0.0 {
            return Err(FvmError::InvalidDiffusivity(diffusivity));
        }
        if !time_step.is_finite() || time_step <= 0.0 {
            return Err(FvmError::InvalidTimeStep(time_step));
        }
        if diffusivity > 0.0 {
            let maximum = mesh.spacing() * mesh.spacing() / (2.0 * diffusivity);
            if time_step > maximum {
                return Err(FvmError::UnstableExplicitStep {
                    maximum,
                    actual: time_step,
                });
            }
        }

        let view = FvmLineMeshView::from_mesh(mesh)?;
        let kernels = FvmKernelSet::default();
        let mut table = ResourceTable::new();
        let state = table.declare_persistent_buffer(host_buffer::<f64>(view.cell_count()));
        let rhs = table.declare_buffer(host_buffer::<f64>(view.cell_count()));
        let scaled_rhs = table.declare_buffer(host_buffer::<f64>(view.cell_count()));
        let next_state = table.declare_buffer(host_buffer::<f64>(view.cell_count()));
        let owner = table.declare_persistent_buffer(host_buffer::<i32>(view.internal_face_count()));
        let neighbour =
            table.declare_persistent_buffer(host_buffer::<i32>(view.internal_face_count()));
        let face_coefficients =
            table.declare_persistent_buffer(host_buffer::<f64>(view.internal_face_count()));
        let cell_volumes = table.declare_persistent_buffer(host_buffer::<f64>(view.cell_count()));
        let diffusivity_id = table.declare_scalar(ScalarValue::F64(diffusivity));
        let time_step_id = table.declare_scalar(ScalarValue::F64(time_step));
        let cell_count = table.declare_scalar(ScalarValue::I64(
            i64::try_from(view.cell_count()).expect("cell count exceeds i64"),
        ));
        let face_count = table.declare_scalar(ScalarValue::I64(
            i64::try_from(view.internal_face_count()).expect("face count exceeds i64"),
        ));

        let mut graph = ExecutionGraphBuilder::new();
        graph.add_kernel(KernelInvocation::new(
            Arc::clone(kernels.laplacian()),
            [
                state,
                owner,
                neighbour,
                face_coefficients,
                cell_volumes,
                rhs,
                cell_count,
                face_count,
            ],
        )?)?;
        graph.add_kernel(KernelInvocation::new(
            Arc::clone(kernels.scale()),
            [rhs, diffusivity_id, scaled_rhs, cell_count],
        )?)?;
        graph.add_kernel(KernelInvocation::new(
            Arc::clone(kernels.explicit_euler()),
            [state, scaled_rhs, time_step_id, next_state, cell_count],
        )?)?;
        graph.add_copy(BufferCopy::new(
            next_state,
            state,
            view.cell_count() * size_of::<f64>(),
        )?)?;

        let plan = ExecutionPlan::new(table, graph.finish())?;
        Ok(Self {
            plan,
            resources: DiffusionResources {
                state,
                rhs,
                scaled_rhs,
                next_state,
                owner,
                neighbour,
                face_coefficients,
                cell_volumes,
                diffusivity: diffusivity_id,
                time_step: time_step_id,
                cell_count,
                face_count,
            },
            view,
            initial_state,
        })
    }

    pub const fn plan(&self) -> &ExecutionPlan {
        &self.plan
    }

    pub const fn resources(&self) -> DiffusionResources {
        self.resources
    }

    pub const fn view(&self) -> &FvmLineMeshView {
        &self.view
    }

    pub fn initialize(&self, store: &mut runtime::ResourceStore) -> Result<(), FvmError> {
        write_slice(store, self.resources.state, &self.initial_state)?;
        write_slice(store, self.resources.owner, self.view.owner())?;
        write_slice(store, self.resources.neighbour, self.view.neighbour())?;
        write_slice(
            store,
            self.resources.face_coefficients,
            self.view.face_coefficients(),
        )?;
        write_slice(store, self.resources.cell_volumes, self.view.cell_volumes())?;
        Ok(())
    }

    pub fn read_state(&self, store: &runtime::ResourceStore) -> Result<Vec<f64>, FvmError> {
        read_slice(store, self.resources.state, self.view.cell_count())
    }
}

fn host_buffer<T>(count: usize) -> BufferSpec {
    BufferSpec::new(count * size_of::<T>(), MemorySpace::Host).with_alignment(64)
}

fn write_slice<T: Copy>(
    store: &runtime::ResourceStore,
    resource: ResourceId,
    values: &[T],
) -> Result<(), FvmError> {
    let buffer = store
        .buffer(resource)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    let bytes = std::mem::size_of_val(values);
    if buffer.bytes() < bytes {
        return Err(FvmError::Runtime(format!(
            "buffer {resource:?} has {} bytes, but {bytes} are required",
            buffer.bytes(),
        )));
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            values.as_ptr().cast::<u8>(),
            buffer.pointer().as_ptr().cast::<u8>(),
            bytes,
        );
    }
    Ok(())
}

fn read_slice<T: Copy>(
    store: &runtime::ResourceStore,
    resource: ResourceId,
    count: usize,
) -> Result<Vec<T>, FvmError> {
    let buffer = store
        .buffer(resource)
        .map_err(|error| FvmError::Runtime(error.to_string()))?;
    let bytes = count * size_of::<T>();
    if buffer.bytes() < bytes {
        return Err(FvmError::Runtime(format!(
            "buffer {resource:?} has {} bytes, but {bytes} are required",
            buffer.bytes(),
        )));
    }
    let source = buffer.pointer().as_ptr().cast::<T>();
    Ok(unsafe { std::slice::from_raw_parts(source, count) }.to_vec())
}

#[cfg(test)]
mod tests {
    use domain::LineDomain;
    use mesh::StructuredLineMesh;
    use runtime::LocalExecutor;

    use crate::{Diffusion1dPlan, register_portable_kernels};

    #[test]
    fn portable_diffusion_step_conserves_mass_with_zero_flux_boundaries() {
        let mesh = StructuredLineMesh::new(LineDomain::along_x(0.0, 1.0).unwrap(), 16).unwrap();
        let mut initial = vec![0.0; mesh.cells()];
        initial[mesh.cells() / 2] = 1.0;
        let before: f64 = initial.iter().sum();
        let diffusivity = 0.01;
        let dt = 0.25 * mesh.spacing().powi(2) / diffusivity;
        let diffusion = Diffusion1dPlan::new(&mesh, initial, diffusivity, dt).unwrap();
        let mut executor = LocalExecutor::new();
        register_portable_kernels(&mut executor).unwrap();
        let mut resources = executor.prepare(diffusion.plan()).unwrap();
        diffusion.initialize(&mut resources).unwrap();
        for _ in 0..10 {
            executor.execute(diffusion.plan(), &mut resources).unwrap();
        }
        let state = diffusion.read_state(&resources).unwrap();
        let after: f64 = state.iter().sum();
        assert!((before - after).abs() < 1.0e-12);
    }
}
