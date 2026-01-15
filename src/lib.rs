use pyo3::prelude::*;

#[pymodule]
fn sr_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_image, m)?)?;
    m.add_function(wrap_pyfunction!(get_available_models, m)?)?;
    m.add_function(wrap_pyfunction!(get_model_info, m)?)?;
    Ok(())
}

#[pyfunction]
fn process_image(
    input_path: &str,
    output_path: &str,
    scale: f32,
    model: &str,
    gpu_id: i32,
    cpu_mode: bool,
) -> PyResult<(bool, String)> {
    Python::attach(|py| {
        let processor = py.import("image.processor")?;
        let result = processor.call_method1(
            "process_image",
            (input_path, output_path, scale, model, gpu_id, cpu_mode),
        )?;
        result.extract()
    })
}

#[pyfunction]
fn get_available_models() -> PyResult<Vec<String>> {
    Python::attach(|py| {
        let processor = py.import("image.processor")?;
        let result = processor.call_method0("get_available_models")?;
        result.extract()
    })
}

#[pyfunction]
fn get_model_info(model: &str) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        let processor = py.import("image.processor")?;
        let result = processor.call_method1("get_model_info", (model,))?;
        Ok(result.into())
    })
}
