mod candidate;
mod dict;
mod dict_compiler;
mod engine;
mod pipeline;
mod schema;
mod trie;
mod user_freq;

use engine::Engine;
use std::io::{self, Write};

fn main() {
    let mut engine = Engine::new();

    println!("输入拼音，例如: nihao，然后回车。输入数字选择候选。输入 empty 退出。");

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

        // Check if input is a number (candidate selection)
        if let Ok(idx) = input.parse::<usize>() {
            if idx >= 1 && idx <= engine.candidates.len() {
                let selected_text = engine.candidates[idx - 1].text.clone();
                println!("已选择: {}", selected_text);
                engine.record_selection(&selected_text);
            } else {
                println!("无效选择");
            }
            println!();
            continue;
        }

        engine.run_pipeline(&input);

        println!("原始输入: {}", input);
        println!("候选结果:");
        for (i, cand) in engine.candidates.iter().enumerate() {
            match &cand.annotation {
                Some(ann) => println!("  {}. {} ({})", i + 1, cand.text, ann),
                None => println!("  {}. {}", i + 1, cand.text),
            }
        }
        println!();
    }

    engine.save_user_freq();
}
