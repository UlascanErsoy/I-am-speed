use chrono;
use serde_json::{Result, Value};
use std::{
    env,
    process::Command,
    fs,
    fs::{File, OpenOptions},
    io::prelude::*,
    str
};


const CSV_HEADERS: &str = "timestamp,host_name,ping,download_speed,upload_speed,error,test_url\n";

fn main() {

    let binary_path = match env::var("SPEED_TEST_PATH") {
        Ok(path) => path,
        Err(_) => panic!("Please set the SPEED_TEST_PATH environment variable")
    };

    let output_path = match env::var("SPEED_TEST_OUTPUT") {
        Ok(path) => path,
        Err(_) => "speed_test.csv".into() 
    };

    let mut file = match fs::metadata(&output_path) {
        Ok(_) => OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(output_path)
                            .expect("Error opening output file"),
        Err(_) => {
            println!("Creating the output csv");
            let mut file = File::create(output_path)
                                .expect("Error while creating file!");
            file.write(CSV_HEADERS.as_bytes()).expect("Failed writing headers to file");
            file
        }
    };


    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("{binary_path} --format json"))
                        .output()
                        .expect("Unable to run command");
            
    
    let timestamp = chrono::offset::Local::now()
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string();

    println!("{:?}", output);

    let stdout = str::from_utf8(&output.stdout)
                        .expect("Failed to parse stdout into string");
    let results: Value = match serde_json::from_str(stdout) {
        Ok(val) => val,
        Err(_) => Value::String(String::from("no msg"))
    };

    if let None = results.get("type") { 

        println!("DETECTED ERR");
        let err_msg = match results.get("error") {
            Some(msg) => msg.as_str().unwrap(),
            None => "no msg"
        };
        file.write(
            format!("{timestamp},,,0,0,{err_msg},\n").as_bytes())
            .expect("While writing error to file");

        return;
    }
    
    let downspeed = results["download"]["bandwidth"].as_f64().unwrap() / 125000_f64; 
    let upspeed = results["upload"]["bandwidth"].as_f64().unwrap() / 125000_f64;
    let ping = &results["ping"]["latency"];
    let hostname = &results["server"]["name"];
    let url = &results["result"]["url"];

    let new_row = format!("{timestamp},{hostname},{ping},{downspeed},{upspeed},,{url}\n");

    file.write(new_row.as_bytes()).expect("Error while writing to file!");
}
