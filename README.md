# ziran-engine

一个使用 Rust 实现的轻量级拼音输入法引擎，架构设计灵感来源于 [librime](https://github.com/rime/librime)。

> 本项目核心代码由 AI 辅助生成。词库数据来源于 [雾凇拼音 (rime-ice)](https://github.com/iDvel/rime-ice)，遵循 GPL-3.0 协议。

## 特性

- **Trie 前缀匹配** — 拼音和英文均使用 Trie 数据结构，DAG 构建 O(n×m)
- **Beam Search 最优切分** — 对数权重评分 + 分段惩罚，词组优先于单字组合
- **多词表加载** — 中文基础/扩展词库 + 英文词库，二进制缓存加速启动
- **中英混合输入** — 拼音输入时候选栏显示英文翻译注释
- **英文前缀匹配** — 输入英文时显示前缀匹配候选
- **用户词频学习** — 本地记录选词偏好，高频词自动提升排序
- **零外部依赖** — 仅使用 Rust 标准库

## 快速开始

```bash
cargo build
cargo run
```

### 词库准备

词库文件需要放在 `data/` 目录下（已被 .gitignore 排除）：

| 文件 | 说明 |
|------|------|
| `base.dict.yaml` | 基础词库（两字词为主） |
| `ext.dict.yaml` | 扩展词库（多音字注音、长词） |
| `others.dict.yaml` | 容错词库（口语读音、异读） |
| `en.dict.yaml` | 英文词库（~20k 单词） |
| `en_ext.dict.yaml` | 英文扩展（缩写、互联网术语） |
| `bilingual.dict.yaml` | 中英对照词典（中文→英文翻译） |
| `default.yaml` | Schema 配置，声明加载哪些词表 |

从 [雾凇拼音](https://github.com/iDvel/rime-ice) 下载词库文件。

### 示例交互

```
> nihao
候选结果:
  1. 你好 (hello)
  2. 拟好

> shijie
候选结果:
  1. 世界 (world)
  2. 时节
  3. 师姐
  ...

> beijing
候选结果:
  1. 北京
  2. 背景

> hello
候选结果:
  1. hello

输入数字选择候选，选词记录会自动保存到 data/user_freq.tsv。
```

## 架构

```
用户输入 → Engine → Pipeline → Candidates
                       │
           ┌───────────┼───────────┐
           ▼           ▼           ▼
       build_dag   beam_search   finalize
           │           │           │
           ▼           ▼           ▼
         Trie      PathState    UserFreq
     (前缀匹配)   (最优路径)    (词频提升)
                                    │
                                    ▼
                              BilingualIndex
                              (中英注释)
```

### 模块说明

| 模块 | 文件 | 职责 |
|------|------|------|
| Engine | `src/engine.rs` | 引擎入口，持有 Pipeline + UserFreq + candidates |
| Pipeline | `src/pipeline.rs` | 核心管线：DAG 构建 → Beam Search → 回溯 → 候选生成 |
| Trie | `src/trie.rs` | 泛型 `Trie<V>`，O(n×m) 前缀匹配 |
| Dict | `src/dict.rs` | 词典加载，构建拼音/英文/中英 Trie |
| DictCompiler | `src/dict_compiler.rs` | 二进制词典缓存（ZIRC 格式） |
| Schema | `src/schema.rs` | YAML Schema 解析 |
| UserFreq | `src/user_freq.rs` | 用户词频记录（本地 TSV） |
| Candidate | `src/candidate.rs` | 候选词结构体（text + score + annotation） |

## 词典格式

### 中文词典（Tab 分隔）

```
词语	拼音1 拼音2 ...	权重
你好	ni hao	332885
```

### 英文词典（Tab 分隔）

```
单词	编码	权重
hello	hello	100
```

### 中英对照词典（Tab 分隔）

```
中文	英文	权重
你好	hello	100
世界	world	100
```

## 与 librime 的对比

| 维度 | librime | ziran-engine |
|------|---------|--------------|
| 语言 | C++ | Rust |
| 词典格式 | 二进制 mmap（.table.bin + .prism.bin） | YAML 文本 + 二进制缓存 |
| 拼音切分 | SyllableGraph（Double-Array Trie） | Trie DAG + Beam Search |
| 评分 | log(weight) + quality_len + 上下文 | log(weight) + 分段惩罚 + 用户频次 |
| 用户学习 | 事务型 UserDict + 时间衰减 | 简单计数 TSV |
| 模糊拼音 | Speller Algebra（编译时展开） | 未实现 |
| 依赖 | boost, yaml-cpp, leveldb 等 | 零外部依赖 |

## License

GPL-3.0

词库数据来源于 [雾凇拼音 (rime-ice)](https://github.com/iDvel/rime-ice)，遵循其原始许可协议。
