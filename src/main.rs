use std::io;
//use std::io::prelude::*;
use clap::Parser;
use std::collections::{HashMap,HashSet,BTreeMap};
use std::collections::hash_map;
use std::collections::btree_map;
use regex::Regex;
use anyhow::Result;
use bounded_vec_deque::BoundedVecDeque;
use std::time::{Instant,Duration};


type MapToCount = HashMap<String,usize>;
type MapToStrings = BTreeMap<usize,HashSet<String>>;

#[derive(Parser)]
#[command(name = "matchtop")]
#[command(author = "Ershaad Basheer <ebasheer@lbl.gov>")]
#[command(version = "0.1")]
#[command(about = "Running count of lines by matching field", long_about = None)]
struct MatchTop {
    #[arg(long,short,default_value_t = String::from(".*"),value_name="REGEX")]
    pattern: String,
    #[arg(id="window",long,short,default_value_t = 1000,value_name="LINES")]
    winsize: usize,
    #[arg(id="interval",long,short,default_value_t = 2,value_name="INTERVAL")]
    interval: u64,
}

fn push_to_window(bvec: &mut BoundedVecDeque<String>, entry: &str) -> Option<String> {
    bvec.push_back(entry.to_string())
}

fn update_count_exitwin(mathash: &mut MapToCount, count: &mut MapToStrings, entry: &str) -> () {
    let matentry = mathash.entry(entry.to_string());
    let mut me = match matentry {
        hash_map::Entry::Occupied(me) => me,
        _ => panic!("hash_map entry not found"),
    };

    let countentry = count.entry(*me.get());
    assert!(matches!(countentry, btree_map::Entry::Occupied(_)));
    let mut ce = match countentry {
        btree_map::Entry::Occupied(ce) => ce,
        _ => panic!("btreemap entry not found"),
    };
    ce.get_mut().remove(entry);
    if ce.get().is_empty() {
        ce.remove(); 
    }
    
    let curr_count = me.get_mut();
    *curr_count -= 1;
    if *curr_count != 0 {
        let countentry = count.entry(*curr_count).or_insert_with(HashSet::new);
        countentry.insert(entry.to_string());
    } else {
        me.remove();
    }
}

fn update_count_enterwin(mathash: &mut MapToCount, count: &mut MapToStrings, entry: &str) -> () {
    let matentry = mathash.entry(entry.to_string()).or_default();

    let countentry = count.entry(*matentry);
    if let btree_map::Entry::Occupied(mut ce) = countentry {
        ce.get_mut().remove(entry);
        if ce.get().is_empty() {
            ce.remove(); 
        }
    }

    *matentry += 1;
    let countentry = count.entry(*matentry).or_insert_with(HashSet::new);
    countentry.insert(entry.to_string());
}

fn main() ->  Result<()> {
    let cli = MatchTop::parse();
    let mut match_queue: BoundedVecDeque<String> = BoundedVecDeque::new(cli.winsize);
    let mut match_count: MapToCount = HashMap::new();
    let mut count_match: MapToStrings = BTreeMap::new();

    println!("=> pattern: {}", cli.pattern);
    println!("=> window size: {} lines", cli.winsize);
    println!("=> interval: {} sec", cli.interval);
    
    let time_intval = Duration::from_secs(cli.interval);
    let mut time_now = Instant::now();

    let re: Regex = Regex::new(&cli.pattern).expect("invalid regex in pattern");
    if re.captures_len() < 2 {
        panic!("pattern must have a capture group"); 
    }

    let mut lines = io::stdin().lines();
    while let Some(Ok(line)) = lines.next() {
        let caps = re.captures(&line);
        match caps {
            Some(caps) => { 
	        if let Some(cap1) = caps.get(1) {
                    if let Some(exit_entry) = push_to_window(&mut match_queue, cap1.as_str()) {
                        update_count_exitwin(&mut match_count, &mut count_match, &exit_entry);
                    }
                    update_count_enterwin(&mut match_count, &mut count_match, cap1.as_str());
                }
            },
            None => (),
        };
        if time_now.elapsed() > time_intval {
            for (key,val) in count_match.iter() {
                println!("{:?} {:?}", key, val);
            }
            time_now = Instant::now();
        }
    }

    Ok(())
}
