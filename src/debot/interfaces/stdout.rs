use serde_json::Value;

pub const STDOUT_ABI: &str = r#"{
	"ABI version": 2,
	"header": ["time"],
	"functions": [
		{
			"name": "print",
			"inputs": [
				{"name":"message","type":"bytes"}
			],
			"outputs": [
			]
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
	]
}"#;


pub struct Stdout {}
impl Stdout {
    fn print(message: &str) {
        println!("{}", message);
    }

    pub fn call(func: &str, args: &Value) {
        match func {
            "print" => {
                let text_vec = hex::decode(args["message"].as_str().unwrap()).unwrap();
                let text = std::str::from_utf8(&text_vec).unwrap();
                Self::print(text);
            },
            _ => panic!("interface function not found"),
        }
    }
}
