use std::fs;
use std::io;
use std::time;
use std::path::Path;
use std::path::PathBuf;
use zip::ZipArchive;
use argh::FromArgs;

#[derive(FromArgs)]
/// Shell into zips
struct Params {
    #[argh(positional)]
    target: PathBuf,
    /// command to run inside zip
    #[argh(option, short= 'c')]
    command: Option<String>,
}


fn main() {
    let params: Params = argh::from_env();
    std::process::exit(zipedit(params))
}

impl Params {
fn target(&self) -> Option<PathBuf> {
    let target = self.target.clone();
    if target.is_absolute() {
        return Some(target);
    }

    let cwd = std::env::current_dir().ok()?;
    let target = cwd.join(target);
    if target.is_file() {
        return Some(target);
    }

    None
}
}

fn zipedit(params: Params) -> i32 {
    let mem_path = Path::new("/tmp/");
    if ! mem_path.is_dir() {
        return 2
    }
    let ts = time::SystemTime::now().duration_since(time::UNIX_EPOCH)
        .and_then(|x| Ok(x.as_secs()))
        .and_then(|x| Ok(x.to_string()))
        .unwrap();

    let work_dir = &mem_path.join(format!("zipedit_{}",ts)); 
    fs::create_dir(work_dir).unwrap();

    let target_file = &params.target().unwrap();
    let zip_file = fs::File::open(target_file).unwrap();

    let mut archive = ZipArchive::new(zip_file).unwrap();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let dest = work_dir.join(file.sanitized_name());

        if file.is_dir() {
            fs::create_dir_all(dest).unwrap();
            continue
        }
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }
        let mut outfile = fs::File::create(dest).unwrap();
        io::copy(&mut file, &mut outfile).unwrap();
    }

    if let Some(cmd) = params.command {
        std::process::Command::new("bash")
            .current_dir(work_dir)
            .arg("-c")
            .arg(cmd)
            .status()
            .unwrap();
    } else {
        let shell = std::env::var("SHELL").unwrap_or("sh".into());
        std::process::Command::new(shell).current_dir(work_dir).status().unwrap();
    }

    fs::remove_dir_all(work_dir).unwrap();

    0
}
