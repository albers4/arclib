use domain::LineDomain;
use finite_volume::{Diffusion1dPlan, register_portable_kernels};
use mesh::StructuredLineMesh;
use runtime::LocalExecutor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let line = LineDomain::along_x(0.0, 1.0)?;
    let mesh = StructuredLineMesh::new(line, 100)?;

    let mut initial = vec![0.0; mesh.cells()];
    for (index, value) in initial.iter_mut().enumerate() {
        let x = (index as f64 + 0.5) * mesh.spacing();
        *value = (-400.0 * (x - 0.5).powi(2)).exp();
    }

    let diffusivity = 0.01;
    let dt = 0.25 * mesh.spacing().powi(2) / diffusivity;
    let diffusion = Diffusion1dPlan::new(&mesh, initial, diffusivity, dt)?;

    let mut executor = LocalExecutor::new();
    register_portable_kernels(&mut executor)?;
    let mut resources = executor.prepare(diffusion.plan())?;
    diffusion.initialize(&mut resources)?;

    for _ in 0..100 {
        executor.execute(diffusion.plan(), &mut resources)?;
    }

    let state = diffusion.read_state(&resources)?;
    println!("center value after 100 steps: {}", state[state.len() / 2]);
    Ok(())
}
