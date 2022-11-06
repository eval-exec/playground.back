use crossbeam::queue::ArrayQueue;
use crossbeam::sync::ShardedLock;
use once_cell::sync::OnceCell;
use std::sync::Arc;
#[derive(Debug, Clone)]
struct MyHouse {
    id: i32,
}

impl Drop for MyHouse {
    fn drop(&mut self) {
        println!("my house {} dropped", self.id);
    }
}

#[derive(Clone)]
struct App {
    house: Arc<ShardedLock<OnceCell<ArrayQueue<MyHouse>>>>,
}

impl T for App {
    fn init() {
        println!("nothing");
    }
}

trait T: Sync + Send {
    fn init();
}
fn main() {
    let app = App {
        house: Default::default(),
    };

    let app2 = app.clone();
    app2.house.read().unwrap().set(ArrayQueue::new(3)).unwrap();

    (0..3).into_iter().for_each(|i| {
        app2.house
            .read()
            .unwrap()
            .get()
            .unwrap()
            .push(MyHouse { id: i })
            .unwrap()
    });

    let poped = app2.house.read().unwrap().get().unwrap().pop();
    println!("popped a house: {:?}", poped);

    println!("take house");
    let _ = app2.house.write().unwrap().take();
    println!("dropped house");

    println!("end of program");
}
