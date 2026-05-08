use std::process::Command;

pub fn run_sclang(scd_path: &str) -> Result<(), String> {
    eprintln!("Rendering audio...");
    let status = Command::new("sclang")
        .arg(scd_path)
        .status()
        .map_err(|e| format!("audio engine not found — run install.sh to set up dependencies: {}", e))?;

    if !status.success() {
        return Err(format!("audio render failed with code: {}", status));
    }
    Ok(())
}

pub fn wav_to_mp3(wav_path: &str, mp3_path: &str) -> Result<(), String> {
    eprintln!("Converting to MP3...");

    let ffmpeg = Command::new("ffmpeg")
        .args(["-y", "-i", wav_path, "-codec:a", "libmp3lame", "-qscale:a", "2", mp3_path])
        .status();

    if let Ok(s) = ffmpeg {
        if s.success() {
            return Ok(());
        }
    }

    let lame = Command::new("lame")
        .args(["--preset", "standard", wav_path, mp3_path])
        .status()
        .map_err(|e| format!("converter not found — run install.sh to set up dependencies: {}", e))?;

    if lame.success() {
        Ok(())
    } else {
        Err("MP3 conversion failed. Run install.sh to set up dependencies.".to_string())
    }
}
