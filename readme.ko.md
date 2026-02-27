![winload](https://socialify.git.ci/VincentZyu233/winload/image?custom_language=Rust&description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# Winload <img src="https://github.com/user-attachments/assets/62fec846-0442-47f6-bbba-78acdc8803ef" height="32px">

> Linux의 `nload`에서 영감을 받은, 네트워크 대역폭 및 트래픽을 실시간으로 모니터링하는 경량 CLI 도구입니다.

> **[📖 English](readme.md)**
> **[📖 简体中文(大陆)](readme.zh-cn.md)**
> **[📖 繁體中文(台灣)](readme.zh-tw.md)**
> **[📖 日本語](readme.jp.md)**
> **[📖 한국어](readme.ko.md)**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/winload)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/winload)

[![Windows x64 | ARM64](https://img.shields.io/badge/Windows-x64_|_ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)
[![Linux x64 | ARM64](https://img.shields.io/badge/Linux-x64_|_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/VincentZyuApps/winload/releases)
[![macOS x64 | ARM64](https://img.shields.io/badge/macOS-x64_|_ARM64-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)
[![Android x64 | ARM64](https://img.shields.io/badge/Android-x64_|_ARM64-3DDC84?style=for-the-badge&logo=android&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)

[![PyPI](https://img.shields.io/badge/PyPI-3776AB?style=for-the-badge&logo=pypi&logoColor=white)](https://pypi.org/project/winload/)
[![npm](https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white)](https://www.npmjs.com/package/winload-rust-bin)

[![Scoop](https://img.shields.io/badge/Scoop-7B4AE2?style=for-the-badge&logo=scoop&logoColor=white)](https://scoop.sh/#/apps?q=%22https%3A%2F%2Fgithub.com%2FVincentZyuApps%2Fscoop-bucket%22&o=false)
[![AUR](https://img.shields.io/badge/AUR-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)](https://aur.archlinux.org/packages/winload-rust-bin)
[![APT](https://img.shields.io/badge/APT-E95420?style=for-the-badge&logo=debian&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)
[![RPM](https://img.shields.io/badge/RPM-CB1626?style=for-the-badge&logo=redhat&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)

> **[📖 빌드 문서](.github/workflows/build.md)**

## 🚀 소개
Winload는 현대적인 터미널 환경에서 직관적이고 시각적인 네트워크 모니터링 기능을 제공합니다. 처음에는 Windows 환경에서 `nload`의 공백을 메우기 위한 도구로 시작되었으나, 현재는 Linux와 macOS까지 지원 범위를 확장했습니다.

## 🙏 감사의 말
Winload는 Roland Riegel의 고전적인 프로젝트인 [nload](https://github.com/rolandriegel/nload)에서 영감을 얻었습니다. 독창적인 아이디어와 훌륭한 사용자 경험을 제공해 준 원작자에게 깊은 감사를 표합니다.

## ✨ 주요 기능
- **두 가지 구현 방식 제공**
	- **Rust 버전**: 빠르고 메모리 안전하며, 단일 정적 바이너리로 제공되어 일상적인 모니터링에 최적화되어 있습니다.
	- **Python 버전**: 구조가 단순하여 프로토타이핑이나 기능 확장, 통합이 용이합니다.
- **교차 플랫폼 지원**: Windows, Linux, macOS (x64 및 ARM64)를 모두 지원합니다.
- **실시간 시각화**: 실시간으로 유입(Incoming) 및 유출(Outgoing) 트래픽 그래프와 처리량 통계를 보여줍니다.
- **미니멀한 UI**: `nload`의 사용성을 계승한 깔끔한 TUI(텍스트 사용자 인터페이스)를 제공합니다.

## 📥 Python 버전 설치
> 💡 **구똄 참고사항**: PyPI 및 GitHub/Gitee 소스 코드만 Python 버전입니다.  
> 모든 패키지 관리자(Scoop, AUR, npm, APT, RPM)는 **Rust 바이너리**를 제공합니다.
### Python (pip)
```bash
pip install winload
```

## 📥 Rust 버전 설치 (권장)
### npm (크로스 플래트폼)
```bash
npm install -g winload-rust-bin
# 또는 npx 를 직접 사용
npx winload-rust-bin
```
> 6가지 사전 컴파일된 바이너리 포함: x86_64 & ARM64, Windows·Linux·macOS 대응.
### Windows (Scoop 이용)
```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop install winload
```

### Arch Linux (AUR):
```bash
paru -S winload-bin
```

### Linux (간편 설치 스크립트)
> Debian/Ubuntu 및 파생 버전(Linux Mint, Pop!_OS, Deepin, UOS 등) 지원 (apt)

> Fedora/RHEL 및 파생 버전(Rocky Linux, AlmaLinux, CentOS Stream 등) 지원 (dnf)
```bash
curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/winload/main/docs/install_scripts/install.sh | bash
```
> 📄 [설치 스크립트 소스 보기](https://github.com/VincentZyuApps/winload/blob/main/docs/install_scripts/install.sh)

<details>
<summary>수동 설치</summary>

**DEB (Debian/Ubuntu):**
```bash
# GitHub Releases에서 최신 .deb 파일을 다운로드합니다.
sudo dpkg -i ./winload_*_amd64.deb
# 또는 apt를 사용하여 의존성을 자동으로 해결하며 설치합니다.
sudo apt install ./winload_*_amd64.deb
```

**RPM (Fedora/RHEL):**
```bash
sudo dnf install ./winload-*-1.x86_64.rpm
```

**또는 [GitHub Releases](https://github.com/VincentZyuApps/winload/releases)에서 바이너리를 직접 다운로드할 수 있습니다.**

</details>

## ⌨️ 사용법

```bash
winload              # 활성화된 모든 네트워크 인터페이스 모니터링
winload -t 200       # 새로고침 간격을 200ms로 설정
winload -d "Wi-Fi"   # 특정 장치 이름으로 시작 (부분 일치 가능)
winload -e           # TUI에 이모지 장식 활성화 🎉
winload --npcap      # 127.0.0.1 루프백 트래픽 캡처 (Windows, Npcap 필요)
```

### 옵션 상세

| 플래그 | 설명 | 기본값 |
|------|-------------|---------|
| `-t`, `--interval <MS>` | 새로고침 간격 (밀리초 단위) | `500` |
| `-a`, `--average <SEC>` | 평균 계산을 위한 윈도우 시간 (초 단위) | `300` |
| `-d`, `--device <NAME>` | 기본 장치 이름 (부분 일치 가능) | — |
| `-e`, `--emoji` | TUI에서 이모지 장식 활성화 🎉 | 비활성 |
| `-U`, `--unicode` | 그래프에 Unicode 블록 문자 사용 (█▓░·) | 비활성 |
| `-u`, `--unit <UNIT>` | 표시 단위: `bit` 또는 `byte` | `bit` |
| `-b`, `--bar-style <STYLE>` | 바 스타일: `fill`, `color`, 또는 `plain` | `fill` |
| `--in-color <HEX>` | 수신 그래프 색상, 16진수 RGB (예: `0x00d7ff`) | Cyan |
| `--out-color <HEX>` | 송신 그래프 색상, 16진수 RGB (예: `0xffaf00`) | Gold |
| `-m`, `--max <VALUE>` | Y축 최대값 고정 (예: `10M`, `1G`, `500K`) | 자동 |
| `-n`, `--no-graph` | 그래프를 숨기고 통계만 표시 | 비활성 |
| `--hide-separator` | 구분선(등호 행) 숨기기 | 비활성 |
| `--no-color` | 모든 TUI 색상 비활성화 (흑백 모드) | 비활성 |
| `--npcap` | **[Windows Rust 버전 전용]** Npcap을 통해 루프백 트래픽 캡처 | 비활성 |
| `--debug-info` | **[Rust 버전 전용]** 네트워크 인터페이스 디버그 정보 출력 후 종료 | — |
| `-h`, `--help` | 도움말 출력 (`--help --emoji`로 이모지 버전 확인 가능!) | — |
| `-V`, `--version` | **[Rust 버전 전용]** 버전 정보 출력 | — |

### 키보드 단축키

| 키 | 동작 |
|-----|--------|
| `←` / `→` 또는 `↑` / `↓` | 네트워크 장치 전환 |
| `=` | 구분선 표시 여부 전환 |
| `c` | 색상 모드 켜기/끄기 전환 |
| `q` / `Esc` | 프로그램 종료 |

## 🪟 Windows 루프백 (127.0.0.1) 안내

Windows는 표준 API를 통해 루프백 트래픽을 보고하지 못하는 구조적 한계가 있습니다. 이는 [Windows 네트워크 스택의 기능적 결함](docs/win_loopback.md)에 기인합니다.

**Windows에서 루프백 트래픽을 모니터링하려면**, `--npcap` 플래그를 사용하십시오:

```bash
winload --npcap
```

이 기능을 사용하려면 [Npcap](https://npcap.com/#download)이 설치되어 있어야 하며, 설치 과정에서 "Support loopback traffic capture" 옵션이 활성화되어 있어야 합니다.

> 이전에는 Windows 자체의 `GetIfEntry` API를 직접 폴링하는 방식을 시도했으나, 루프백 인터페이스의 카운터는 항상 0으로 나타났습니다. 루프백 가상 인터페이스 뒤에는 데이터를 집계할 NDIS 드라이버가 존재하지 않기 때문입니다. 따라서 해당 코드 경로는 현재 제거되었습니다.

> 📖 Windows 루프백 문제에 대한 기술적인 상세 내용은 [docs/win_loopback.md](docs/win_loopback.md)를 참조하십시오.

Linux 및 macOS에서는 별도의 설정 없이 루프백 트래픽 모니터링이 기본적으로 작동합니다.

## 🖼️ 미리보기
#### Python 버전 미리보기
![docs/preview-py.png](docs/preview-py.png)

#### Rust 버전 미리보기
![docs/preview-rust.png](docs/preview-rust.png)
