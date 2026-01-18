use pyo3::prelude::*;
use std::path::Path;

const MODEL_PREFIXES: &[&str] = &["REALCUGAN", "REALESRGAN", "REALSR", "WAIFU2X"];

pub fn process_image(
    input: &str,
    output: &str,
    scale: f32,
    model: &str,
    gpu_id: i32,
    cpu: bool,
    model_path: Option<&str>,
) -> Result<(bool, String), String> {
    Python::attach(|py| py_process_image(py, input, output, scale, model, gpu_id, cpu, model_path))
        .map_err(|e| e.to_string())
}

pub fn list_models() -> Result<String, String> {
    Python::attach(|py| py_list_models(py)).map_err(|e| e.to_string())
}

#[pyfunction]
#[pyo3(name = "process_image")]
fn py_process_image(
    py: Python,
    input: &str,
    output: &str,
    scale: f32,
    model: &str,
    gpu_id: i32,
    cpu: bool,
    model_path: Option<&str>,
) -> PyResult<(bool, String)> {
    let sr = py.import("sr_vulkan.sr_vulkan")?;

    let effective_model_path = model_path
        .map(|p| p.to_string())
        .or_else(|| std::env::var("SR_MODEL_PATH").ok());

    if let Some(ref path) = effective_model_path {
        sr.call_method1("setModelPath", (path,))?;
    }

    let init_result: i32 = sr.call_method0("init")?.extract()?;

    let mut use_cpu = cpu;
    if init_result < 0 {
        use_cpu = true;
    }

    let init_set_result: i32 = if use_cpu {
        let cpu_num: i32 = sr.call_method0("getCpuCoreNum")?.extract()?;
        sr.call_method1("initSet", (-1, cpu_num))?.extract()?
    } else {
        sr.call_method1("initSet", (gpu_id,))?.extract()?
    };

    if init_set_result < 0 {
        return Ok((false, "Initialization failed".to_string()));
    }

    let normalized_model = format!(
        "model_{}",
        model.to_lowercase().replace("-", "_").replace(" ", "_")
    );

    let models = py.import("sr_vulkan.sr_vulkan")?;
    let mut model_id: Option<i32> = None;

    {
        let attr = normalized_model.to_uppercase();
        if let Ok(id) = models.getattr(&attr) {
            model_id = id.extract().ok();
        }
    }

    if model_id.is_none() {
        for attr in models.dir()? {
            let attr_name: String = attr.extract()?;
            if attr_name.starts_with("MODEL_") {
                if let Ok(id) = models.getattr(&attr_name) {
                    if let Ok(id_val) = id.extract::<i32>() {
                        let model_name = attr_name.replace("MODEL_", "").to_lowercase();
                        if model == model_name
                            || model.contains(&model_name)
                            || model_name.contains(model)
                        {
                            model_id = Some(id_val);
                            break;
                        }
                    }
                }
            }
        }
    }

    let model_id = match model_id {
        Some(id) => id,
        None => return Ok((false, format!("Unknown model: {}", model))),
    };

    process_image_inner(py, input, output, scale, model_id)
}

fn process_image_inner(
    py: Python,
    input: &str,
    output: &str,
    scale: f32,
    model_id: i32,
) -> PyResult<(bool, String)> {
    let sr = py.import("sr_vulkan.sr_vulkan")?;

    if !Path::new(input).exists() {
        return Ok((false, format!("Input file not found: {}", input)));
    }

    let data = std::fs::read(input)?;

    let add_result: i32 = sr
        .call_method1("add", (data, model_id, 1, scale))?
        .extract()?;

    if add_result <= 0 {
        let error: String = sr.call_method0("getLastError")?.extract()?;
        return Ok((false, format!("Failed to add task: {}", error)));
    }

    let mut wait_count = 0;
    let max_wait = 60;

    while wait_count < max_wait {
        let info = sr.call_method1("load", (0,))?;
        if info.is_none() {
            std::thread::sleep(std::time::Duration::from_millis(500));
            wait_count += 1;
            continue;
        }

        let tuple: (Py<PyAny>, String, i32, f32) = info.extract()?;
        if tuple.0.is_none(py) {
            std::thread::sleep(std::time::Duration::from_millis(500));
            wait_count += 1;
            continue;
        }

        let output_data: Vec<u8> = tuple.0.extract(py)?;
        let out_format = tuple.1;
        let result_id = tuple.2;
        let tick = tuple.3;

        let output_file = format!("{}.{}", result_id, out_format);
        std::fs::write(&output_file, &output_data)?;
        std::fs::rename(&output_file, output)?;

        sr.call_method0("stop")?;
        return Ok((true, format!("{:.2}", tick)));
    }

    sr.call_method0("stop")?;
    Ok((false, "Processing timeout".to_string()))
}

#[pyfunction]
#[pyo3(name = "list_models")]
fn py_list_models(py: Python) -> PyResult<String> {
    let sr = py.import("sr_vulkan.sr_vulkan")?;
    let mut categories: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for attr in sr.dir()? {
        let attr_name: String = attr.extract()?;
        if attr_name.starts_with("MODEL_") {
            if let Ok(id) = sr.getattr(&attr_name) {
                let _: i32 = id.extract()?;
                let model_name = attr_name.replace("MODEL_", "").to_lowercase();

                for prefix in MODEL_PREFIXES {
                    if model_name.starts_with(&prefix.to_lowercase()) {
                        let clean_name = model_name
                            .replace(&prefix.to_lowercase(), "")
                            .trim_start_matches('_')
                            .to_string();
                        if !clean_name.is_empty() {
                            categories
                                .entry(prefix.to_string())
                                .or_default()
                                .push(clean_name);
                        }
                        break;
                    }
                }
            }
        }
    }

    let mut output = String::new();
    for prefix in MODEL_PREFIXES {
        if let Some(models) = categories.get(prefix as &str) {
            output.push_str(&format!("{}:\n", prefix));
            for model in models {
                output.push_str(&format!("  - {}\n", model));
            }
            output.push('\n');
        }
    }

    Ok(output)
}

#[pymodule]
#[pyo3(name = "sr_bindings")]
fn sr_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_process_image, m)?)?;
    m.add_function(wrap_pyfunction!(py_list_models, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}

#[pyfunction]
#[pyo3(name = "main")]
fn main() -> PyResult<()> {
    Ok(())
}
