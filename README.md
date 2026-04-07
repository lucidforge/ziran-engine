# ziran-engine

一个使用 Rust 实现的轻量级输入法引擎，架构设计灵感来源于 [librime](https://github.com/rime/librime)。

## 特性

### 当前已实现

- [x] 模块化架构：Engine / Context / Pipeline / Segmentor / Translator
- [x] 基础拼音输入（整串匹配）
- [x] 文件词库加载（`data/dict.txt`）
- [x] 多候选输出（一个拼音对应多个候选词）
- [x] 候选排序（按 score 降序）
- [x] 未匹配词原样输出

### 计划中

- [ ] 真正的拼音分段器（如 `xi'an` → `ni` + `hao`）
- [ ] 拼音词典 Trie 树 / 前缀匹配
- [ ] 多音节组合翻译（`nihao` → `你 好`）
- [ ] 用户词频学习
- [ ] Bigram / N-gram 语言模型
- [ ] 单元测试覆盖
- [ ] YAML 词库格式
- [ ] TOML 配置文件
- [ ] IMKit / TSF / Fcitx 前端对接

## 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 编译运行

```bash
cargo build
cargo run
```

### 示例交互

```
输入拼音，例如: nihao，然后回车。输入 empty 退出。
> nihao
原始输入: nihao
候选结果:
  1. 你好
  2. 你号

> zhongguo
原始输入: zhongguo
候选结果:
  1. 中国
  2. 种果

> abc
原始输入: abc
候选结果:
  1. abc
```

## 架构设计

```
用户输入
    │
    ▼
┌─────────────┐
│   Engine    │  引擎入口，管理 Context 和 Pipeline
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Context    │  共享上下文：raw_input → segments → candidates
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Pipeline   │────▶│  Segmentor   │────▶│ Translator   │
└─────────────┘     └──────────────┘     └──────┬───────┘
                                                │
                                                ▼
                                         ┌──────────────┐
                                         │   Sort       │
                                         └──────┬───────┘
                                                │
                                                ▼
                                         候选列表输出
```

### 模块说明

| 模块 | 文件 | 职责 |
|------|------|------|
| Engine | `src/engine.rs` | 引擎入口，协调各组件 |
| Context | `src/context.rs` | 共享上下文，存储输入、分段、候选 |
| Pipeline | `src/pipeline.rs` | 处理管线，串联分段→翻译→排序 |
| Segmentor | `src/segmentor.rs` | 拼音分段器 |
| Translator | `src/translator.rs` | 翻译器，将拼音转为汉字 |
| Segment | `src/segment.rs` | 分段数据结构 |
| Candidate | `src/candidate.rs` | 候选词数据结构 |

## 词库格式

词库文件位于 `data/dict.txt`，每行格式为：

```
拼音 词语
```

示例：

```
nihao 你好
nihao 你号
zhongguo 中国
beijing 北京
```

支持一个拼音对应多个词语，按出现顺序决定初始权重。

## 与 librime 的关系

本项目架构设计灵感来源于 [librime](https://github.com/rime/librime)，采用 Rust 重新实现。

与 librime 的差异：

| 维度 | librime | ziran-engine |
|------|---------|--------------|
| 语言 | C++ | Rust |
| 定位 | 生产级输入法引擎 | 学习/实验性质 |
| 架构 | 复杂插件系统 | 极简模块化 |
| 依赖 | boost, yaml-cpp, leveldb 等 | 零外部依赖（当前） |

## 路线图

### Phase 1：核心引擎完善（当前阶段）

- [ ] 实现真正的拼音分段算法
- [ ] 支持多音节组合翻译
- [ ] 用户词频学习

### Phase 2：工程化

- [ ] 单元测试
- [ ] YAML 词库格式
- [ ] TOML 配置文件
- [ ] 日志系统

### Phase 3：前端对接

- [ ] 命令行交互优化（逐键输入、退格、选词）
- [ ] IMKit (macOS)
- [ ] TSF (Windows)
- [ ] Fcitx/IBus (Linux)

## License

MIT
