"""Python图片超分辨率处理模块，使用PyO3暴露给Rust调用。"""

import os
import time
from typing import Dict, List, Optional, Tuple

from sr_vulkan import sr_vulkan as sr

MODEL_PREFIXES = ["REALCUGAN", "REALCUGAN_SE", "REALESRGAN", "REALSR", "WAIFU2X"]

_LIBRARY_MODELS_CACHE: Optional[Dict[str, int]] = None


def get_library_models() -> Dict[str, int]:
    """获取模型缓存，延迟加载"""
    global _LIBRARY_MODELS_CACHE
    if _LIBRARY_MODELS_CACHE is None:
        _LIBRARY_MODELS_CACHE = _get_all_models_from_library()
    return _LIBRARY_MODELS_CACHE


def _get_all_models_from_library() -> Dict[str, int]:
    """从sr_vulkan库动态获取所有模型"""
    models = {}
    for attr in dir(sr):
        if attr.startswith("MODEL_"):
            try:
                value = getattr(sr, attr)
                if isinstance(value, int):
                    name = attr.replace("MODEL_", "").lower()
                    models[name] = value
            except Exception:
                pass
    return models


def normalize_model_name(name: str) -> str:
    """规范化模型名称"""
    name = name.lower()
    name = name.replace("-", "_").replace(" ", "_")
    if not name.startswith("model_"):
        name = "model_" + name
    return name


def suppress_output(func, *args, **kwargs):
    """抑制C库的stdout/stderr输出"""
    old_stdout_fd = os.dup(1)
    old_stderr_fd = os.dup(2)
    devnull = os.open(os.devnull, os.O_WRONLY)
    os.dup2(devnull, 1)
    os.dup2(devnull, 2)
    os.close(devnull)
    try:
        result = func(*args, **kwargs)
    finally:
        os.dup2(old_stdout_fd, 1)
        os.dup2(old_stderr_fd, 2)
        os.close(old_stdout_fd)
        os.close(old_stderr_fd)
    return result


def find_model_id(name: str) -> Optional[int]:
    """根据名称查找模型ID"""
    models = get_library_models()
    normalized = normalize_model_name(name)
    if normalized in models:
        return models[normalized]
    for key, value in models.items():
        if normalized in key or key in normalized:
            return value
    return None


class ImageProcessor:
    """图片超分辨率处理器"""

    def __init__(self, gpu_id: int = 0, cpu_mode: bool = False):
        """初始化处理器"""
        self.gpu_id = gpu_id
        self.cpu_mode = cpu_mode
        self.initialized = False

    def init(self) -> bool:
        """初始化GPU/CPU"""
        sts = suppress_output(sr.init)

        if sts < 0:
            self.cpu_mode = True

        if self.cpu_mode:
            cpu_num = sr.getCpuCoreNum()
            sts = suppress_output(sr.initSet, -1, cpu_num)
        else:
            sts = suppress_output(sr.initSet, self.gpu_id)

        self.initialized = sts >= 0
        return self.initialized

    def process(
        self,
        input_path: str,
        output_path: str,
        scale: float = 2.0,
        model: str = "realesrgan_x4plus",
        tile_size: int = 400,
        output_format: str = "webp",
    ) -> Tuple[bool, str]:
        """处理图片超分辨率"""
        if not self.initialized:
            if not self.init():
                return False, "初始化失败"

        if not os.path.exists(input_path):
            return False, f"输入文件不存在: {input_path}"

        model_id = find_model_id(model)
        if model_id is None:
            return False, f"未知模型: {model}"

        with open(input_path, "rb") as f:
            data = f.read()

        back_id = 1

        add_result = suppress_output(
            sr.add,
            data,
            model_id,
            back_id,
            scale,
            tileSize=tile_size,
            format=output_format,
        )
        if add_result <= 0:
            return False, "添加任务失败"

        max_wait = 60
        wait_count = 0

        while wait_count < max_wait:
            info = sr.load(0)
            if info:
                new_data, out_format, result_id, tick = info
                if new_data:
                    output_file = f"{result_id}.{out_format}"
                    with open(output_file, "wb") as f:
                        f.write(new_data)
                    os.rename(output_file, output_path)
                    suppress_output(sr.stop)
                    return True, f"{tick:.2f}"
            time.sleep(1)
            wait_count += 1

        suppress_output(sr.stop)
        return False, "处理超时"


def get_all_model_names() -> List[str]:
    """获取所有可用模型名称"""
    return list(get_library_models().keys())


def process_image(
    input_path: str,
    output_path: str,
    scale: float = 2.0,
    model: str = "realesrgan_x4plus",
    gpu_id: int = 0,
    cpu_mode: bool = False,
    model_path: Optional[str] = None,
) -> Tuple[bool, str]:
    """处理单张图片（PyO3导出函数）"""
    if model_path:
        sr.setModelPath(model_path)
    processor = ImageProcessor(gpu_id=gpu_id, cpu_mode=cpu_mode)
    return processor.process(
        input_path=input_path, output_path=output_path, scale=scale, model=model
    )


def get_model_info(model: str) -> dict:
    """获取模型信息"""
    model_id = find_model_id(model)
    if model_id is not None:
        return {"name": model, "id": model_id, "description": "sr_vulkan 模型"}
    return {"name": model, "id": None, "description": "未知模型"}


def get_model_categories_formatted() -> str:
    """获取格式化后的模型分类字符串"""
    categories = {}
    for name in get_library_models().keys():
        for prefix in MODEL_PREFIXES:
            if prefix.lower() in name:
                if prefix not in categories:
                    categories[prefix] = []
                categories[prefix].append(name.replace("model_", ""))
                break
    lines = []
    for prefix in MODEL_PREFIXES:
        if prefix in categories:
            lines.append(f"{prefix}:")
            for model in categories[prefix]:
                lines.append(f"  - {model}")
            lines.append("")
    return "\n".join(lines)
