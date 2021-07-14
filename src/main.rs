use std::fs;
use std::io;
use std::time;
use std::hash::{Hash,Hasher};
use std::path::{PathBuf, Path};
use zip::ZipArchive;
use argh::FromArgs;
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use time::SystemTime;

#[derive(FromArgs)]
/// Shell into zips
struct Params {
    #[argh(positional)]
    target: PathBuf,
    /// command to run inside zip
    #[argh(option, short= 'c')]
    command: Option<String>,
}


fn main() -> Result<()> {
    color_eyre::install()?;
    let params = get_params()?;
    zipedit(params)?;
    Ok(())
}

fn get_params() -> Result<Params> {
    let mut params: Params = argh::from_env();
    if params.target.is_absolute() {
        return Ok(params);
    }

    let cwd = std::env::current_dir()?;
    let target = cwd.join(params.target);
    if target.is_file() {
        params.target = target;
        return Ok(params);
    }
    Err(eyre!("No such file \"{}\"", target.to_string_lossy()))
}

fn zipedit(params: Params) -> Result<()>{
    let work_dir = get_temp_work_dir(&params.target.to_string_lossy())?;

    unpack_to_dir(&params.target, &work_dir)?;

    if let Some(cmd) = params.command {
        std::process::Command::new("bash")
            .current_dir(&work_dir)
            .arg("-c")
            .arg(cmd)
            .status();
    } else {
        let shell = std::env::var("SHELL").unwrap_or("sh".into());
        std::process::Command::new(shell).current_dir(&work_dir).status();
    }

    fs::remove_dir_all(work_dir)?;

    Ok(())
}

fn get_temp_work_dir(path: &str) -> Result<PathBuf> {
    let tmp_path = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|x| x.as_millis())?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);

    Ok(tmp_path.join(format!("zipedit_{}_{:x}",timestamp, hasher.finish())))
}

fn unpack_to_dir(zip: &Path, dest_dir: &Path) -> Result<()> {
    let zip_file = fs::File::open(zip)?;

    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let dest = dest_dir.join(file.sanitized_name());

        if file.is_dir() {
            fs::create_dir_all(dest)?;
            continue
        }
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut outfile = fs::File::create(dest)?;
        io::copy(&mut file, &mut outfile)?;
    }
    Ok(())
}
