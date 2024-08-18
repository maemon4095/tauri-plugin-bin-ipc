mod command;

pub use command::command;

fn x() {
    struct X;
    trait Y {
        const N: &'static str;
    }
    impl Y for X {
        const N: &'static str = "X";
    }

    match "X" {
        <X as Y>::N => {}
        _ => {}
    }
}
