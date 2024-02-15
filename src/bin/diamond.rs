use diamond_types::list::*;
use diamond_types::list::encoding::*;

fn main() {
    let mut oplog = OpLog::new();
    let fred = oplog.get_or_create_agent_id("fred");
    let _time = oplog.add_insert(fred, 0, "🥕abc🥕");
    let _time2 = dbg!(oplog.add_delete_without_content(fred, 0..2)); // Delete the 'b'

    let branch = Branch::new_at_tip(&oplog);
    // Equivalent to let mut branch = Branch::new_at_local_version(&oplog, oplog.get_local_version());
    assert_eq!("bc🥕", branch.content().to_string());

    let options = EncodeOptions::default();
    let change = oplog.encode(options.clone());

    let mut oplog_remote = OpLog::new();
    let local_version = oplog_remote.decode_and_add(&change).unwrap();

    let branch = oplog_remote.checkout(&local_version);
    assert_eq!("bc🥕", branch.content().to_string());

    // let time = oplog.add_insert(fred, 3, "dc");

    // let times = &[time];
    // let change = oplog.encode_from(options, times);
    // let local_version = oplog_remote.decode_and_add(&change).unwrap();

    // let branch = oplog_remote.checkout(&local_version);
    // assert_eq!("bc🥕", branch.content().to_string());
}
