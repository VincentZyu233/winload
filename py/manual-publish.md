# 手动发布 winload (Python) 到 PyPI

> 目标: `pip install winload` 或 `uv pip install winload` 一键安装，装完直接命令行敲 `winload` 就能用。

---

## 0. 前置条件

- 一个 [PyPI 账号](https://pypi.org/account/register/)（正式发布）
- 建议也注册 [TestPyPI](https://test.pypi.org/account/register/)（先拿来练手）
- 已安装 `uv`（你本地已经有了）

---

## 1. 项目结构改造

当前是散装 `.py`，PyPI 需要打成 **包 (package)**。目标结构:

```
py/
├── pyproject.toml          ← 新增: 项目元数据 + 构建配置
├── README.md               ← 新增: 给 PyPI 页面用的说明
├── LICENSE                 ← 新增/复用: 仓库根目录已有的话软链或复制
└── src/
    └── winload/            ← 把所有 .py 挪进来，变成一个包
        ├── __init__.py     ← 新增
        ├── __main__.py     ← 新增: 支持 python -m winload
        ├── main.py
        ├── collector.py
        ├── graph.py
        ├── stats.py
        └── ui.py
```

### 1.1 创建目录 & 移动文件

```powershell
cd D:\aaaStuffsaaa\from_git\github\winload\py

# 创建 src layout
mkdir -p src/winload

# 移动源码
mv collector.py graph.py main.py stats.py ui.py src/winload/
```

### 1.2 创建 `src/winload/__init__.py`

```python
"""winload - Windows Network Load Monitor"""

__version__ = "0.1.0"
```

### 1.3 创建 `src/winload/__main__.py`

让 `python -m winload` 能直接跑:

```python
from winload.main import main

main()
```

### 1.4 修复包内 import

所有源文件里的裸 import 要改成包内相对导入或绝对导入。例如:

| 改前                                      | 改后                                            |
| ----------------------------------------- | ----------------------------------------------- |
| `from collector import Collector`          | `from winload.collector import Collector`        |
| `from stats import StatisticsEngine, ...`  | `from winload.stats import StatisticsEngine, ...`|
| `from graph import render_graph, ...`      | `from winload.graph import render_graph, ...`    |

> **提示**: 也可以用 **相对导入** (`from .collector import ...`)，效果一样。

---

## 2. 创建 `pyproject.toml`

```toml
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "winload"
version = "0.1.0"
description = "A lightweight, real-time CLI tool for monitoring network bandwidth, inspired by nload"
readme = "README.md"
license = "MIT"
requires-python = ">=3.8"
authors = [
    { name = "VincentZyu233" },
]
keywords = ["network", "monitor", "nload", "tui", "bandwidth"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Environment :: Console :: Curses",
    "Operating System :: Microsoft :: Windows",
    "Operating System :: POSIX :: Linux",
    "Operating System :: MacOS",
    "Programming Language :: Python :: 3",
    "Topic :: System :: Networking :: Monitoring",
]
dependencies = [
    "psutil>=5.9",
    "windows-curses>=2.0; sys_platform == 'win32'",
]

[project.scripts]
winload = "winload.main:main"

[project.urls]
Homepage = "https://github.com/VincentZyu233/winload"
Repository = "https://github.com/VincentZyu233/winload"
```

> **关键点**: `[project.scripts]` 这一行会让 pip 自动生成 `winload` / `winload.exe` 命令行入口。

---

## 3. 写个 README.md (给 PyPI 页面)

可以简单写，也可以从仓库根目录的 `readme.md` 摘抄:

```markdown
# winload

A lightweight, real-time CLI tool for monitoring network bandwidth, inspired by Linux's nload.

## Install

pip install winload

## Usage

winload              # 监控所有活跃网卡
winload -t 200       # 刷新间隔 200ms
winload -d "Wi-Fi"   # 指定默认设备
```

---

## 4. 本地测试安装

在正式上传之前，先本地验证能不能装、能不能跑:

```powershell
cd D:\aaaStuffsaaa\from_git\github\winload\py

# 用 uv 装到当前 venv（可编辑模式，改代码立刻生效）
uv pip install -e .

# 试试能不能跑
winload
# 或者
python -m winload
```

如果正常弹出 TUI 界面，说明包结构没问题 ✅

---

## 5. 构建分发包

```powershell
# 安装构建工具
uv pip install build

# 构建 sdist + wheel
python -m build
```

构建完会在 `dist/` 下生成两个文件:

```
dist/
├── winload-0.1.0.tar.gz        ← sdist (源码包)
└── winload-0.1.0-py3-none-any.whl  ← wheel (安装快)
```

---

## 6. 上传到 TestPyPI（先练手）

```powershell
# 安装上传工具
uv pip install twine

# 上传到 TestPyPI
twine upload --repository testpypi dist/*
```

会提示输入用户名密码。**推荐用 API Token**:
1. 去 https://test.pypi.org/manage/account/#api-tokens 创建 Token
2. 用户名填 `__token__`，密码填 token 值（`pypi-` 开头那串）

上传成功后，测试安装:

```powershell
# 从 TestPyPI 安装
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ winload
```

> `--extra-index-url` 是因为 `psutil` 等依赖在正式 PyPI 上，TestPyPI 上没有。

---

## 7. 正式上传到 PyPI 🎉

确认 TestPyPI 一切正常后:

```powershell
twine upload dist/*
```

同样推荐 API Token:
1. 去 https://pypi.org/manage/account/#api-tokens 创建
2. 用户名 `__token__`，密码填 token

上传完成！现在任何人都可以:

```bash
pip install winload
# 或
uv pip install winload
```

---

## 8. 后续版本更新

1. 改 `pyproject.toml` 和 `__init__.py` 里的 `version`
2. **删掉旧的 dist/**:
   ```powershell
   Remove-Item dist/* -Recurse -Force
   ```
3. 重新构建 + 上传:
   ```powershell
   python -m build
   twine upload dist/*
   ```

---

## 🔧 可选: 用 `.pypirc` 省去每次输密码

在 `~/.pypirc`（Windows 上是 `C:\Users\你的用户名\.pypirc`）写入:

```ini
[distutils]
index-servers =
    pypi
    testpypi

[pypi]
username = __token__
password = pypi-你的正式PyPI-Token

[testpypi]
repository = https://test.pypi.org/legacy/
username = __token__
password = pypi-你的TestPyPI-Token
```

这样 `twine upload` 就不用每次手动输了。

---

## ⚡ 速查: 完整流程一把梭

```powershell
# 1. 整理好 src layout + pyproject.toml（只需做一次）

# 2. 本地验证
uv pip install -e .
winload

# 3. 构建
python -m build

# 4. 上传 (TestPyPI 先试, 没问题再正式)
twine upload --repository testpypi dist/*   # 练手
twine upload dist/*                          # 正式

# 5. 验证
pip install winload
winload
```

搞定 🎉
