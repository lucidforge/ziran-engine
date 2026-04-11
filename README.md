# ziran-engine

一个使用 Rust 实现的轻量级拼音输入法引擎，架构设计灵感来源于 [librime](https://github.com/rime/librime)。

> 本项目核心代码由 AI 辅助生成。词库数据来源于 [雾凇拼音 (rime-ice)](https://github.com/iDvel/rime-ice)，遵循 GPL-3.0 协议。

## 特性

### 已实现

- [x] 模块化架构：Engine / Context / Pipeline / Segmentor / Translator
- [x] 拼音自动切分（DP 最优匹配，考虑词库权重）
- [x] 多词表加载（base、ext、others、en、en_ext）
- [x] 短语优先匹配 + 单字组合生成
- [x] 英文单词输入
- [x] 候选排序（按词库权重降序）
- [x] 未匹配词原样输出

### 已知问题

DP 切分在个别场景下会选错。比如 `zhongguo` → "中过"而非"中国"，原因是单字权重（`中` 768万 + `过` 730万）远高于二字词（`中国` 50万），DP 贪心选了大权重路径。这个问题需要在 DP 评分函数里加入词长惩罚或平滑因子，让短语比单字组合更优先。

### 计划中

- [ ] 修复 DP 评分函数（词长惩罚）
- [ ] Trie 前缀匹配 / 输入时补全
- [ ] 用户词频学习
- [ ] Bigram / N-gram 语言模型
- [ ] 多音字消歧
- [ ] 单元测试覆盖
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
  2. 拟好

> shijie
原始输入: shijie
候选结果:
  1. 世界
  2. 时节
  3. 师姐
  4. 视界
  ...

> hello
原始输入: hello
候选结果:
  1. hello
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
| Segmentor | `src/segmentor.rs` | 拼音分段器（DP 最优匹配） |
| Translator | `src/translator.rs` | 翻译器，多词表索引 + 短语优先 + 组合生成 |
| Segment | `src/segment.rs` | 分段数据结构 |
| Candidate | `src/candidate.rs` | 候选词数据结构 |

## 词库

词库数据来源于 [雾凇拼音 (rime-ice)](https://github.com/iDvel/rime-ice)，存放于 `data/` 目录：

| 文件 | 说明 |
|------|------|
| `base.dict.yaml` | base 基础词库（两字词为主） |
| `ext.dict.yaml` | 扩展词库（多音字注音、长词） |
| `others.dict.yaml` | 容错词库（口语读音、异读） |
| `en.dict.yaml` | 英文词库（~20k 单词） |
| `en_ext.dict.yaml` | 英文扩展（缩写、互联网术语） |

> `tencent.dict.yaml`（腾讯词向量）因无拼音注音，暂未加载。

### 词库格式（中文）

Tab 分隔三列：

```
词语	拼音1 拼音2 ...	权重
```

### 词库格式（英文）

Tab 分隔两列：

```
单词	单词
```

## 与 librime 的关系

本项目架构设计灵感来源于 [librime](https://github.com/rime/librime)，采用 Rust 重新实现。

与 librime 的差异：

| 维度 | librime | ziran-engine |
|------|---------|--------------|
| 语言 | C++ | Rust |
| 定位 | 生产级输入法引擎 | 学习/实验性质 |
| 架构 | 复杂插件系统 | 极简模块化 |
| 依赖 | boost, yaml-cpp, leveldb 等 | 零外部依赖 |

## 路线图

### Phase 1：核心引擎完善（当前阶段）

- [x] 拼音自动切分
- [x] 多词表加载
- [x] DP 最优切分（基础版，有已知评分函数问题待修复）
- [ ] 修复 DP 评分函数

### Phase 2：工程化

- [ ] 单元测试
- [ ] 日志系统
- [ ] 配置文件

### Phase 3：前端对接

- [ ] 命令行交互优化（逐键输入、退格、选词）
- [ ] IMKit (macOS)
- [ ] TSF (Windows)
- [ ] Fcitx/IBus (Linux)

## License

GPL-3.0

词库数据来源于 [雾凇拼音 (rime-ice)](https://github.com/iDvel/rime-ice)，遵循其原始许可协议。
