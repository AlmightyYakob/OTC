use std::fs;
use std::path::PathBuf;

use nom::{
    bytes::complete::{take, take_until},
    IResult,
};

#[derive(Debug, Clone)]
pub struct InvalidScriptError;
pub fn get_script_contents(path: &PathBuf) -> Result<String, InvalidScriptError> {
    let pathstr = path.to_str().unwrap();
    let data =
        fs::read_to_string(path).expect(format!("Unable to read file: {}", pathstr).as_str());

    let prescriptres: IResult<&str, &str> = take_until("<script>")(data.as_str());
    if prescriptres.is_err() {
        return Err(InvalidScriptError);
    }

    // Take <script>
    let prescript: IResult<&str, &str> = take(8usize)(prescriptres.unwrap().0);
    let remaining = prescript.unwrap().0;

    // Grab remaining data
    let scriptres: IResult<&str, &str> = take_until("</script>")(remaining);
    if scriptres.is_err() {
        return Err(InvalidScriptError);
    }

    return Ok(scriptres.unwrap().1.to_string());
}
