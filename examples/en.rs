enum Enum {
    String(String),
}

#[derive(Debug, PartialEq, Eq)]
enum AorB {
    A,
    B
}

fn a_or_b(e: &Enum) -> AorB {
    if let Enum::String(s) = e {
        if s == "A" {

        }
        match s {
            "A" => println!(""),
            "B" => println!(""),
            _ => panic!("")
        }

    }
    match e {
        Enum::String(s) if s == "A" => AorB::A,
        Enum::String(s) if s == "B" => AorB::B,
        _ => panic!()
    }
}

fn main() {
    let a = Enum::String("A".to_string());
    let b = Enum::String("B".to_string());

    if let Enum::String(s) = &a {
        match s {
            "A" => println!(""),
            "B" => println!(""),
            _ => panic!("")
        }
    }
    assert_eq!(a_or_b(&a), AorB::A);
    assert_eq!(a_or_b(&b), AorB::B);
}
