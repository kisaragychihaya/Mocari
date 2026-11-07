//! Regression test for the eyeball evaluation gap (issue: `ParamEyeBallX/Y`
//! are bound but deform nothing on some rigs).
//!
//! Validation of the repro embedded in pet/docs/research/mocari-issue-eyeball.md
//! §3, upgraded to assert the fix. It needs a real model directory via
//! `EYEBALL_MODEL_DIR` (a dir containing a `.model3.json`); without it the
//! test skips, since the affected model is not redistributable.
use mocari::runtime::ModelRuntime;

fn vertex_positions(runtime: &ModelRuntime) -> Vec<Vec<[f32; 2]>> {
    runtime
        .meshes()
        .iter()
        .map(|m| m.vertices().iter().map(|v| v.position()).collect())
        .collect()
}

fn max_delta(a: &[Vec<[f32; 2]>], b: &[Vec<[f32; 2]>]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| {
            x.iter()
                .zip(y)
                .map(|(u, v)| (u[0] - v[0]).abs().max((u[1] - v[1]).abs()))
                .fold(0.0f32, f32::max)
        })
        .fold(0.0f32, f32::max)
}

fn model3_in(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.to_string_lossy().ends_with("model3.json") && p.to_string_lossy() != "model3.json"
    })
}

#[test]
fn eyeball_params_deform_geometry() {
    let Ok(dir) = std::env::var("EYEBALL_MODEL_DIR") else {
        eprintln!("EYEBALL_MODEL_DIR not set; skipping (model not redistributable)");
        return;
    };
    let Some(model3) = model3_in(std::path::Path::new(&dir)) else {
        eprintln!("no .model3.json in {dir}; skipping");
        return;
    };
    let mut model = mocari::assets::load_model_runtime(&model3).unwrap();
    let runtime = model.runtime_mut();

    for id in ["ParamEyeBallX", "ParamEyeBallY", "ParamEyeLOpen"] {
        let delta = probe(runtime, id).unwrap_or(0.0);
        eprintln!("{id}: max vertex delta = {delta:.6}");
        assert!(
            delta > 1e-4,
            "{id} is bound but deforms nothing (max delta {delta})"
        );
    }
}

fn probe(runtime: &mut ModelRuntime, id: &str) -> Option<f32> {
    let index = runtime.parameter_index(id)?;
    let min = runtime.parameter_minimum_by_index(index).unwrap();
    let max = runtime.parameter_maximum_by_index(index).unwrap();
    let default = runtime.parameter_default_by_index(index).unwrap();
    runtime.update_meshes()?;
    let base = vertex_positions(runtime);
    let mut best = 0.0f32;
    for v in [min, max] {
        runtime.set_parameter_by_index(index, v);
        runtime.update_meshes()?;
        best = best.max(max_delta(&base, &vertex_positions(runtime)));
    }
    runtime.set_parameter_by_index(index, default);
    runtime.update_meshes()?;
    Some(best)
}
