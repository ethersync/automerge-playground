#![allow(unused, dead_code)]
use automerge::{
    iter::Keys, patches::TextRepresentation, sync::{Message, State as SyncState, SyncDoc}, transaction::Transactable, ActorId, AutoCommit, Change, ChangeHash, ObjType, legacy::OpType, Patch, PatchLog, ReadDoc, Value
};
use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile, Text};
use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    // automerge_text();
    // automerge_example();
    // autosurgeon_example()
    // let _ = basic_patchlog_sync_example();
    // basic_conflict_demo();
    // merge_independent_heads();
    ethersync_file_history(&args[1]);
}

fn automerge_text() {
    let mut doc = AutoCommit::new();
    let one_char_per_position = doc
        .put_object(automerge::ROOT, "example1", ObjType::Text)
        .unwrap();
    let three_char_per_position = doc
        .put_object(automerge::ROOT, "example2", ObjType::Text)
        .unwrap();

    // let _ = doc.update_text(&one_char_per_position, "foobar");
    // alternative initialization:
    doc.insert(&one_char_per_position, 0, "f").unwrap();
    doc.insert(&one_char_per_position, 1, "o").unwrap();
    doc.insert(&one_char_per_position, 2, "o").unwrap();
    doc.insert(&one_char_per_position, 3, "b").unwrap();
    doc.insert(&one_char_per_position, 4, "a").unwrap();
    doc.insert(&one_char_per_position, 5, "r").unwrap();

    assert_eq!(doc.length(&one_char_per_position), 6);
    assert_eq!(doc.text(&one_char_per_position).unwrap(), "foobar");

    doc.splice_text(&one_char_per_position, 5, 1, "z").unwrap();
    assert_eq!(doc.length(&one_char_per_position), 6);
    assert_eq!(doc.text(&one_char_per_position).unwrap(), "foobaz");

    doc.insert(&three_char_per_position, 0, "foo").unwrap();
    doc.insert(&three_char_per_position, 1, "bar").unwrap();

    // the content still renders the same
    assert_eq!(doc.length(&three_char_per_position), 6);
    assert_eq!(doc.text(&three_char_per_position).unwrap(), "foobar");

    // but splice_text rather works on the "two 'three character' elements",
    // at least in some terms:
    // This is consistent to me and works as expected:
    doc.splice_text(&three_char_per_position, 3, 3, "xyz")
        .unwrap();
    // but: you can't every only replace a part of "foo" or "bar".
    // in fact all these splice_text calls would produce a similar result:
    // doc.splice_text(&three_char_per_position, 3, 1, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 3, 2, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 4, 1, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 4, 2, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 4, 3, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 5, 1, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 5, 2, "xyz").unwrap();
    // doc.splice_text(&three_char_per_position, 5, 3, "xyz").unwrap();
    assert_eq!(doc.length(&three_char_per_position), 6);
    assert_eq!(doc.text(&three_char_per_position).unwrap(), "fooxyz");

    // Furthermore, splice_text seems to do some form of insert that "removes" my weird
    // three-char-groupings, because in a "second round" of splice_text it behaves differently
    // so the underlying representation has changed
    doc.splice_text(&three_char_per_position, 3, 1, "abc")
        .unwrap();
    assert_eq!(doc.length(&three_char_per_position), 8);
    assert_eq!(doc.text(&three_char_per_position).unwrap(), "fooabcyz");

    let s = "🥕字👩‍❤️‍💋‍👩";
    doc.update_text(&one_char_per_position, s).unwrap();
    // where's the 10th character coming from?? => Display Heart as Emojii (<fe0f>)
    // for c in s.chars() {
    //     println!("{:?}", c);
    // }
    // println!("{}", s.chars().count());
    assert_eq!(doc.length(&one_char_per_position), 10);
}

/// Reproducing what is mentioned in https://docs.rs/automerge/latest/automerge/#conflicts
fn basic_conflict_demo() {
    // AutoCommit implements the ReadDoc trait
    let mut doc = AutoCommit::new();
    let contact = doc
        .put_object(automerge::ROOT, "contact", ObjType::Map)
        .unwrap();

    let mut doc2 = doc.fork().with_actor(ActorId::random());
    let contact2 = doc2.get(automerge::ROOT, "contact").unwrap().unwrap().1;

    doc2.put(&contact2, "name", "Bob").unwrap();
    doc.put(&contact, "name", "Alice").unwrap();

    doc.merge(&mut doc2).unwrap();
    dbg!(doc.get(&contact, "name").unwrap()); // winning value picked "random, but deterministically"
    dbg!(doc.get_all(&contact, "name").unwrap()); // here, we have two elements
}

fn merge_independent_heads() {
    // AutoCommit implements the ReadDoc trait
    let mut doc = AutoCommit::new();
    let contact = doc
        .put_object(automerge::ROOT, "contact", ObjType::Map)
        .unwrap();

    let mut doc2 = AutoCommit::new();
    let contact2 = doc2
        .put_object(automerge::ROOT, "contact", ObjType::Map)
        .unwrap();

    doc2.put(&contact2, "name", "Bob").unwrap();
    doc.put(&contact, "name", "Alice").unwrap();

    dbg!(doc.get_heads());
    doc.merge(&mut doc2).unwrap();
    dbg!(doc.get_heads());

    dbg!(doc.get_all(&contact, "name").unwrap()); // here, we have only a single element
    dbg!(doc.get_all(automerge::ROOT, "contact").unwrap()); // here, we have two elements
                                                            // => this merge happens on the level of the map!! :-o

    doc.put(&contact, "name", "Eve").unwrap();

    dbg!(doc.get_all(&contact, "name").unwrap());
    dbg!(doc.get_all(automerge::ROOT, "contact").unwrap());

    let contact3 = doc
        .put_object(automerge::ROOT, "contact", ObjType::Map)
        .unwrap();

    dbg!(doc.get_all(automerge::ROOT, "contact").unwrap());
}

fn automerge_example() {
    // AutoCommit implements the ReadDoc trait
    let mut doc = AutoCommit::new();

    // `put_object` creates a nested object in the root key/value map and
    // returns the ID of the new object, in this case a list.
    let contacts = doc
        .put_object(automerge::ROOT, "contacts", ObjType::List)
        .unwrap();

    // Now we can insert objects into the list
    let alice = doc.insert_object(&contacts, 0, ObjType::Map).unwrap();

    // Finally we can set keys in the "alice" map
    doc.put(&alice, "name", "Alice").unwrap();
    doc.put(&alice, "email", "alice@example.com").unwrap();

    // Create another contact
    let bob = doc.insert_object(&contacts, 1, ObjType::Map).unwrap();
    doc.put(&bob, "name", "Bob").unwrap();
    doc.put(&bob, "email", "bob@example.com").unwrap();

    // Now we save the address book, we can put this in a file
    let saved: Vec<u8> = doc.save();

    // Load the document on the first device and change alices email
    let mut doc1 = AutoCommit::load(&saved).unwrap();
    let contacts = match doc1.get(automerge::ROOT, "contacts").unwrap() {
        Some((automerge::Value::Object(ObjType::List), contacts)) => contacts,
        _ => panic!("contacts should be a list"),
    };
    let alice = match doc1.get(&contacts, 0).unwrap() {
        Some((automerge::Value::Object(ObjType::Map), alice)) => alice,
        _ => panic!("alice should be a map"),
    };
    doc1.put(&alice, "email", "alicesnewemail@example.com")
        .unwrap();

    // Load the document on the second device and change bobs name
    let mut doc2 = AutoCommit::load(&saved).unwrap();
    let contacts = match doc2.get(automerge::ROOT, "contacts").unwrap() {
        Some((automerge::Value::Object(ObjType::List), contacts)) => contacts,
        _ => panic!("contacts should be a list"),
    };
    let bob = match doc2.get(&contacts, 1).unwrap() {
        Some((automerge::Value::Object(ObjType::Map), bob)) => bob,
        _ => panic!("bob should be a map"),
    };
    doc2.put(&bob, "name", "Robert").unwrap();

    // Finally, we can merge the changes from the two devices
    doc1.merge(&mut doc2).unwrap();
    let bobsname: Option<automerge::Value> = doc1.get(&bob, "name").unwrap().map(|(v, _)| v);
    assert_eq!(
        bobsname,
        Some(automerge::Value::Scalar(Cow::Owned("Robert".into())))
    );

    let alices_email: Option<automerge::Value> = doc1.get(&alice, "email").unwrap().map(|(v, _)| v);
    assert_eq!(
        alices_email,
        Some(automerge::Value::Scalar(Cow::Owned(
            "alicesnewemail@example.com".into()
        )))
    );

    let blogpost = doc
        .put_object(automerge::ROOT, "blogpost", ObjType::Text)
        .unwrap();
    dbg!(&blogpost);

    let text = doc.insert(&blogpost, 0, "foobar").unwrap();
    // What does this do?
    // let _ = doc.insert(&blogpost, 1, "xvar").unwrap();

    dbg!(&doc.length(&blogpost));

    let _ = dbg!(doc.splice_text(&blogpost, 3, 1, "z").unwrap());

    dbg!(&doc.length(&blogpost));

    dbg!(&text);

    // Is the index relevant here? Somehow it is, putting it out of bounds makes this None
    dbg!(doc.get(&blogpost, 0).unwrap());
    dbg!(doc.text(&blogpost).unwrap());
    dbg!(doc.object_type(&blogpost).unwrap());
    // dbg!(doc.get(&blogpost, 2).unwrap());
    // dbg!(doc.get(&blogpost, 42).unwrap());
    // dbg!(moritz);

    // let cursor = doc.get_cursor(&moritz, 0, None).unwrap();
    // dbg!(cursor.to_bytes());
    println!("Example finished.");
}

fn autosurgeon_example() {
    #[derive(Debug, Reconcile, Hydrate)]
    struct Quote {
        text: Text,
    }
    let mut doc = automerge::AutoCommit::new();
    let quote = Quote {
        text: "glimmers".into(),
    };
    reconcile(&mut doc, &quote).unwrap();

    // Fork and make changes to the text
    let mut doc2 = doc.fork().with_actor(ActorId::random());
    let mut quote2: Quote = hydrate(&doc2).unwrap();
    quote2.text.splice(0, 0, "All that ");
    let end_index = quote2.text.as_str().char_indices().last().unwrap().0;
    quote2.text.splice(end_index + 1, 0, " is not gold");
    reconcile(&mut doc2, &quote2).unwrap();

    // Concurrently modify the text in the original doc
    let mut quote: Quote = hydrate(&doc).unwrap();
    let m_index = quote.text.as_str().char_indices().nth(3).unwrap().0;
    quote.text.splice(m_index, 2, "tt");
    reconcile(&mut doc, quote).unwrap();

    // Merge the changes
    doc.merge(&mut doc2).unwrap();

    let quote: Quote = hydrate(&doc).unwrap();
    assert_eq!(quote.text.as_str(), "All that glitters is not gold");
}

fn basic_patchlog_sync_example() -> Result<(), Box<dyn Error>> {
    let mut peer1 = AutoCommit::new();
    let the_text = peer1.put_object(automerge::ROOT, "text", ObjType::Text)?;
    let _ = peer1.update_text(&the_text, "foobar");

    // Create a state to track our sync with peer2
    let mut peer1_state = SyncState::new();
    // Generate the initial message to send to peer2, unwrap for brevity
    let message1to2 = peer1
        .sync()
        .generate_sync_message(&mut peer1_state)
        .unwrap();

    // We receive the message on peer2. We don't have a document at all yet
    // so we create one
    let mut peer2 = automerge::AutoCommit::new();
    // We don't have a state for peer1 (it's a new connection), so we create one
    let mut peer2_state = SyncState::new();

    let mut patch_log = PatchLog::active(TextRepresentation::String);
    let _ = peer2.sync().receive_sync_message_log_patches(
        &mut peer2_state,
        message1to2.clone(),
        &mut patch_log,
    );
    let patches = peer2.make_patches(&mut patch_log);
    dbg!(patches);

    // Now receive the message from peer 1
    // peer2
    //     .sync()
    //     .receive_sync_message(&mut peer2_state, message1to2)?;

    // Now we loop, sending messages from one to two and two to one until
    // neither has anything new to send

    loop {
        let two_to_one = peer2.sync().generate_sync_message(&mut peer2_state);
        if let Some(message) = two_to_one.as_ref() {
            println!("two to one");
            peer1
                .sync()
                .receive_sync_message(&mut peer1_state, message.clone())?;
        }
        let one_to_two = peer1.sync().generate_sync_message(&mut peer1_state);
        if let Some(message) = one_to_two.as_ref() {
            println!("one to two");
            let _ = peer2.sync().receive_sync_message_log_patches(
                &mut peer2_state,
                message.clone(),
                &mut patch_log,
            );
            let patches = peer2.make_patches(&mut patch_log);
            dbg!(patches);
            // peer2
            //     .sync()
            //     .receive_sync_message(&mut peer2_state, message.clone())?;
        }
        if two_to_one.is_none() && one_to_two.is_none() {
            break;
        }
    }

    let the_text_p2 = peer2.get(automerge::ROOT, "text")?.map(|(_, o)| o).unwrap();
    assert_eq!(peer2.text(&the_text_p2)?, "foobar");

    Ok(())
}

fn ethersync_file_history(doc_path: &str) {
    let bytes = fs::read(doc_path).unwrap();
    let mut doc = AutoCommit::load(&bytes).unwrap();

    //dbg!(&doc.keys(automerge::ROOT).collect::<Vec<_>>());

    let file_map = doc.get(automerge::ROOT, "files").unwrap().unwrap().1;

    //dbg!(&doc.keys(&file_map).collect::<Vec<_>>());
    let heads = doc.get_heads();
    let mut current_head = heads[0];
    for _ in 0..10 {
        let change = doc.get_change_by_hash(&current_head).unwrap();
        println!("* {}", summarize(change));
        let parents = change.deps();
        current_head = parents[0];
    }

    /*
    dbg!(&heads);
    let changes = doc.get_changes(&[]);
    dbg!(changes.len());
    let mut hashes = vec![];
    let mut i = 0;
    for change in &changes {
        let h = change.hash();
        if i > 10 && i < 100 {
            dbg!(&change.decode());
        }
        i += 1;

        if i >= 100 {
            return;
        }
        // dbg!(&change.hash());
        hashes.push(h);
    }
    let mut number_of_documents: usize = 0;
    let mut prev_keys: HashSet<String> = HashSet::new();

    for h in hashes {
        let keys = doc.keys_at(&file_map, &[h]);
        let c = keys.count();
        if c != number_of_documents {
            number_of_documents = c;
            let keys_new = doc.keys_at(&file_map, &[h]).collect::<HashSet<_>>();
            let added: Vec<_> = keys_new.difference(&prev_keys).collect();
            if !added.is_empty() {
                println!("Newly added documents in {h}:");
                dbg!(added);
            }
            let removed: Vec<_> = prev_keys.difference(&keys_new).collect();
            if !removed.is_empty() {
                println!("Removed documents in {h}:");
                dbg!(removed);
            }
            prev_keys = keys_new;
        }
    }
    */
}

/*
- Anzahl der Ops pro Typ Op
*/
fn summarize(change: &Change) -> String {
    let expanded_change = change.decode();
    let ops = &expanded_change.operations;
    match ops.len() {
        0 => {"zero ops ???".into()},
        1 => {
            // diplay nicely
            match ops[0].action {
                OpType::Put(value) => format!("put {value}"),
                OpType::Delete => "delete".into(),
                _ => "some other op".into(),
            }
        }
        _ => {
            // count types
            for ops in ops {
                dbg!(&ops.action);
            }
            format!("{} ops", expanded_change.operations.len())
        }
    }
}
