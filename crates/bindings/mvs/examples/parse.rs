use quick_xml::events::Event;



fn main() {
    let src = std::fs::read_to_string("sys/mvs/nodes.xml").unwrap();

    let mut reader = quick_xml::Reader::from_str(&src);
    reader.trim_text(true);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(e) => {
                match e {
                    Event::Eof => {
                        break;
                    },
                    Event::Start(e) => {
                        let e = e.local_name();
                        println!("start {:?}", e);
                    },
                    Event::End(e) => {
                        e.name();
                        println!("end {:?}", e);
                        panic!();
                    },
                    _ => {}
                }
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}