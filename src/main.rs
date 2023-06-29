use std::collections::HashMap;
use clipboard_win::{Clipboard, Getter, Setter, formats::Unicode};


struct Request {
    method: String,
    url: String,
    body: String,
    headers: HashMap<String, String>
}

impl Request {
    fn load(text: &str) -> Request {
        let lines = text.lines().collect::<Vec<_>>();
        let first_line = &lines[0];

        let first_line_chunks = first_line.split(" ").collect::<Vec<_>>();
        let method = first_line_chunks[0].to_string();

        let host_header = text.lines()
            .find(|line| line.starts_with("Host: "))
            .expect("host is not in request headers");

        let path = &first_line_chunks[1];
        let domain = host_header.split("Host: ").collect::<Vec<_>>()[1];

        let url = format!("https://{}{}", domain, path);
        let mut headers: HashMap<String, String> = HashMap::new();

        for line in lines.iter().skip(1) {
            if !line.starts_with("Host: ") {
                let first_colon_pos = line.find(":").unwrap();
                let (key, value) = (line[..first_colon_pos].to_string(), line[first_colon_pos+2..].to_string());
                headers.insert(key, value);
            }
        }

        return Request{
            method,
            url,
            body: String::new(),
            headers,
        };
    }

    fn build_golang_net_http(&self) -> String {
        let mut result = String::new();
        let content_type = match self.headers.get("Content-Type") {
            Some(ct) => ct.clone(),
            _ => String::new(),
        };

        let data_line = {
            if self.body.is_empty() { 
                String::new() 
            } else if content_type.contains("application/x-www-form-urlencoded") {
                let data_map = load_urlencoded_data(&self.body);
                let mut payload_code = String::from("var payload url.Values = map[string][]string{\n");
                for (key, value) in data_map.iter() {
                    let mut line = String::from("\t");
                    line.push_str(&format!(r#""{}": {{`{}`}},"#, key, value));
                    line.push_str("\r\n");
                    payload_code.push_str(&line);
                }
                payload_code.push_str("}\r\n");
                payload_code.push_str("content := strings.NewReader(payload.Encode())\r\n");
                payload_code
            } else if content_type.contains("application/json") {
                let json_object: serde_json::Value = serde_json::from_str(&self.body).unwrap(); 
                format!("content := strings.NewReader(`{}`)\r\n", serde_json::to_string_pretty(&json_object).unwrap())
            } else {
                format!("content := strings.NewReader(`{}`)\r\n", self.body) 
            }
        };

        result.push_str(&data_line);

        let content_variable = match result.is_empty() {
            true => "nil",
            false => "content",
        };

        let first_line = format!(r#"req, _ := http.NewRequest("{}", "{}", {})"#, self.method, self.url, content_variable);
        result.push_str(&first_line);
        result.push_str("\r\n");
        
        let mut header_code = String::new();
        header_code.push_str("req.Header = map[string][]string{\n");
        for (key, value) in self.headers.iter() {
            let mut line = String::from("\t");
            line.push_str(&format!(r#""{}": {{`{}`}},"#, key, value));
            line.push_str("\r\n");
            header_code.push_str(&line);
        }
        header_code.push_str("}\r\n");

        result.push_str(&header_code);
        return result;
    }

    fn build_python_requests(&self) -> String {
        let mut result = String::new();

        let content_type = match self.headers.get("Content-Type") {
            Some(ct) => ct.clone(),
            _ => String::new(),
        };

        let data_line = {
            if self.body.is_empty() { 
                String::new() 
            } else if content_type.contains("application/x-www-form-urlencoded") {
                let data_map = load_urlencoded_data(&self.body);
                let json_data = serde_json::to_string_pretty(&data_map).unwrap();
                format!("payload = {}", json_data)
            } else if content_type.contains("application/json") {
                // sorry i dont feel like parsing True/False & None lol, just write in a real programming language or parse it yourself in ur ide lmao 
                format!("payload = {}", self.body)
            } else {
                format!("payload = '{}'\r\n", self.body) 
            }
        };

        result.push_str(&data_line);
        result.push_str("\r\n");

        let mut header_code = String::new();
        header_code.push_str("headers = {\r\n");
        for (key, value) in self.headers.iter() {
            let mut line = String::from("\t");
            line.push_str(&format!(r#"'{}': '{}',"#, key, value));
            line.push_str("\r\n");
            header_code.push_str(&line);
        }
        header_code.push_str("}\r\n");
        result.push_str(&header_code);

        let request_code = format!("response = requests.{}(url='{}', headers=headers{})\r\n",
            self.method.to_lowercase(), self.url, if self.body.is_empty() {""}  else if content_type.contains("application/json") {", json=payload"} else { ", data=payload" }
        );
        result.push_str(&request_code);

        return result;
    }

    fn build_golang_fasthttp(&self) -> String {
        let mut result = String::new();

        let content_type = match self.headers.get("Content-Type") {
            Some(ct) => ct.clone(),
            _ => String::new(),
        };

        let first_line = "req := fasthttp.AcquireRequest()\r\n";
        let second_line = "defer fasthttp.ReleaseRequest(req)\r\n\r\n";
        let method_line = format!("req.Header.SetMethod(\"{}\")\r\n\r\n", self.method);

        let mut header_code = String::new();
        for (key, value) in self.headers.iter() {
            let mut line = format!(r#"req.Header.Add("{}",`{}`)"#, key, value);
            line.push_str("\r\n");
            header_code.push_str(&line);
        }

        let data_line = {
            if self.body.is_empty() { 
                String::new() 
            } else if content_type.contains("application/x-www-form-urlencoded") {
                let data_map = load_urlencoded_data(&self.body);
                let mut payload_code = String::from("var payload url.Values = map[string][]string{\n");
                for (key, value) in data_map.iter() {
                    let mut line = String::from("\t");
                    line.push_str(&format!(r#""{}": {{`{}`}},"#, key, value));
                    line.push_str("\r\n");
                    payload_code.push_str(&line);
                }
                payload_code.push_str("}\r\n");
                payload_code.push_str("req.AppendBodyString(payload.Encode())\r\n\r\n");
                payload_code
            } else if content_type.contains("application/json") {
                let json_object: serde_json::Value = serde_json::from_str(&self.body).unwrap(); 
                format!("req.AppendBodyString(`{}`)\r\n\r\n", serde_json::to_string_pretty(&json_object).unwrap())
            } else {
                format!("req.AppendBodyString(`{}`)\r\n\r\n", self.body) 
            }
        };

        result.push_str(&first_line);
        result.push_str(&second_line);
        result.push_str(&method_line);
        result.push_str(&data_line);
        result.push_str(&header_code);
        
        return result;
    }
}

fn main() {
    let text = {
        let mut text = String::new();
        let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
        match Unicode.read_clipboard(&mut text) {
            Ok(_) => {},
            Err(e) => panic!("There was an error reading from the clipboard: {:?}", e),
        };
        text
    };
    
    let mut request = Request::load(&text);
    if request.method != "GET" {
        println!("Please press enter when you have copied the request body or type '*' and press enter for nil body...");
        let text = read_input::<String>().unwrap();

        match text {
            text if !text.is_empty() && text.as_bytes()[0] == b'*' => {},
            _ => {
                request.body = {
                    let mut text = String::new();
                    let _clip = Clipboard::new_attempts(10).expect("Open clipboard");
                    match Unicode.read_clipboard(&mut text) {
                        Ok(_) => {},
                        Err(e) => panic!("There was an error reading from the clipboard: {:?}", e),
                    };
                    text
                };
            }
        }
    }

    println!("[0] Go (net/http)");
    println!("[1] Python (requests)");
    println!("[2] Go (github.com/valyala/fasthttp)");
    eprint!("Please pick an option >> ");
    
    let built_request = match read_input::<String>().unwrap().as_bytes()[0] {
        b'0' => request.build_golang_net_http(),
        b'1' => request.build_python_requests(),
        b'2' => request.build_golang_fasthttp(),
        _ => panic!("invalid option")
    };

    let _clip = Clipboard::new_attempts(10).expect("clipboard error");
    Unicode.write_clipboard(&built_request).unwrap();

    println!("Copied to clipboard.");
}

fn load_urlencoded_data(data: &str) -> HashMap<String, String> {
    let mut m: HashMap<String, String> = HashMap::new();
    let key_values = data.split("&").collect::<Vec<_>>();

    for key_value in key_values.iter() {
        let slice = key_value.split("=").collect::<Vec<_>>();
        let (key, value) = (slice[0].to_string(), urlencoding::decode(slice[1]).unwrap().to_string());
        m.insert(key, value);
    }

    return m;
}

fn read_input<T: std::str::FromStr>
    () -> Result<T, Box<dyn std::error::Error>> {

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();

    match input.parse() {
        Ok(value) => Ok(value),
        Err(_) => Err("could not be parsed".into())
    }
}
