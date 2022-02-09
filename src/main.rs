use std::env;
use std::process::Command;

mod tree;
use tree::{Kind, Target, Tree};

#[derive(Debug)]
enum Error {
    Args,
    Retrieve,
    Parse,
    Neighbor,
    Command,
    Message,
}

fn main() -> Result<(), Error> {
    let args: Box<[String]> = env::args().collect();
    let targets = parse_args(&args).ok_or(Error::Args)?;
    let mut get_tree = Command::new("swaymsg");
    get_tree.arg("-t").arg("get_tree");
    let input = get_tree.output().ok().ok_or(Error::Retrieve)?;

    let mut tree: Tree = serde_json::from_slice(input.stdout.as_slice())
        .ok()
        .ok_or(Error::Parse)?;
    tree.reform();
    let neighbor = tree.neighbor(&targets).ok_or(Error::Neighbor)?;

    let mut cmd = Command::new("swaymsg");
    let focus_cmd = neighbor.focus_command().ok_or(Error::Command)?;
    cmd.arg(focus_cmd);
    cmd.spawn()
        .and_then(|mut p| p.wait())
        .ok()
        .ok_or(Error::Message)?;

    Ok(())
}

fn parse_args(args: &[String]) -> Option<Box<[Target]>> {
    if args.len() < 2 {
        return None;
    }
    let targets = args[1..].iter().map(|arg| {
        let split = arg.split_once('-')?;
        let kind = match split.0 {
            "split" => Some(Kind::Split),
            "group" => Some(Kind::Group),
            "float" => Some(Kind::Float),
            "output" => Some(Kind::Output),
            _ => None,
        }?;
        if let [dir, wrap] = split.1.as_bytes() {
            let (backward, vertical) = match dir {
                0x75 => Some((true, true)),
                0x64 => Some((false, true)),
                0x6c => Some((true, false)),
                0x72 => Some((false, false)),
                _ => None,
            }?;
            let wrap = match wrap {
                0x77 => Some(true),
                0x73 => Some(false),
                _ => None,
            }?;
            Some(Target {
                kind,
                backward,
                vertical,
                wrap,
            })
        } else {
            None
        }
    });
    targets.collect()
}
