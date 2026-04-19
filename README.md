# ThaiLLM SDK & Tooling

A comprehensive suite of SDKs and tools for interacting with Thai Large Language Models (LLMs). This project provides multi-language support and proxy utilities to simplify the integration of Thai LLM services into various applications.

## Project Structure

This repository is organized into several modules, each catering to different development needs:

| Module | Description | Key Features |
|---|---|---|
| [**Dart SDK**](./dart) | A high-level client for Dart and Flutter. | Simple API, Multi-turn chat, Error handling, support for Typhoon, OpenThaiGPT, Pathumma, etc. |
| [**Rust Agent**](./rust) | An agentic framework built with `adk-rust`. | Custom `ThaiLLM` model trait, Tool support (Weather, Filesystem), CLI & API server modes. |
| [**LiteLLM Proxy**](./litellm) | OpenAI-compatible proxy layer. | LiteLLM integration for Typhoon, allows using standard OpenAI clients with Thai LLMs. |

## 🚀 Getting Started

### Dart/Flutter SDK

Ideal for mobile and web applications.

```bash
cd dart
dart pub get
```

*See [Dart README](./dart/README.md) for usage examples.*

### Rust Agentic Framework

Ideal for building autonomous agents with tool-use capabilities.

```bash
cd rust
# Set your THAILLM_API_KEY in .env
cargo run
```

*See [Rust README](./rust/README.md) for more details.*

### LiteLLM Proxy

Provides an OpenAI-compatible API for ThaiLLM models.

```bash
cd litellm
pip install -r requirements.txt
python typhoon_proxy.py
```

*See [LiteLLM README](./litellm/README.md) for proxy configuration.*

## 🛠️ Supported Models

The SDKs are designed to work with various Thai LLM providers, including:

- **Typhoon** (by SCB 10X)
- **OpenThaiGPT**
- **Pathumma** (by NECTEC)
- **KBTG**

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details (if applicable).
