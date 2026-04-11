mod candidate;
mod context;
mod dict;
mod engine;
mod pipeline;
mod schema;
mod segment;
mod segmentor;
mod translator;

use engine::Engine;
use std::io::{self, Write};

fn main() {
    let mut engine = Engine::new();

    println!("输入拼音，例如: nihao，然后回车。输入 empty 退出。");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let input = line.trim().to_string();
        if input.is_empty() || input == "empty" {
            break;
        }

        engine.context.raw_input = input.clone();
        engine.run_pipeline();

        println!("原始输入: {}", input);
        println!("候选结果:");
        for (i, cand) in engine.context.candidates.iter().enumerate() {
            println!("  {}. {}", i + 1, cand.text);
        }
        println!();
    }
}
