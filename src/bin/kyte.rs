use kyte::{Compose, Delta, Transform};

fn main() {
    let before = Delta::new().insert("Hello World".to_owned(), ());

    let alice = Delta::new().retain(5, ()).insert(",".to_owned(), ());
    let bob = Delta::new().retain(11, ()).insert("!".to_owned(), ());

    let composed_1 = before
        .clone()
        .compose(alice.clone())
        .compose(dbg!(alice.clone().transform(bob.clone(), true)));
    let composed_2 = before
        .clone()
        .compose(bob.clone())
        .compose(dbg!(bob.clone().transform(alice.clone(), false)));

    assert_eq!(composed_1, composed_2);
    println!("{:?}", composed_1);
}
