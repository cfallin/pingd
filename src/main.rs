extern crate oping;
extern crate iron;
extern crate rusqlite;
extern crate crossbeam;
extern crate time;

use std::env;
use std::sync::{Arc, Mutex};
use oping::Ping;
use std::time::{Instant, SystemTime, Duration, UNIX_EPOCH};
use std::thread::sleep;
use iron::{Request, IronResult, Response, status};
use iron::middleware::Handler;
use iron::headers::ContentType;
use iron::mime::{Mime, TopLevel, SubLevel};
use std::path::Path;

type DBConn = Arc<Mutex<rusqlite::Connection>>;

struct Pinger {
    host: String,
    db: DBConn,
}

impl Pinger {
    pub fn new<S: Into<String>>(host: S, db: DBConn) -> Pinger {
        Pinger {
            host: host.into(),
            db: db,
        }
    }

    // Sends one ping, and returns a (Unix time, Option<latency_in_ms>)
    // tuple. The second element is None if no ping reply was received.
    fn do_ping(&self) -> (u64, Option<f64>) {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut p = Ping::new();
        p.set_timeout(1.0).unwrap();
        p.add_host(&self.host).unwrap();
        let mut iter = p.send().unwrap();
        let response = iter.next().unwrap();
        (time,
         if response.dropped > 0 {
            None
        } else {
            Some(response.latency_ms)
        })
    }

    fn record_result(&self, result: (u64, Option<f64>)) {
        let l = self.db.lock().unwrap();
        let mut s = l.prepare("insert into pings(host, t, missing, latency_ms) values(?, ?, ?, ?)")
                     .unwrap();
        let unix_time: i64 = result.0 as i64;
        let (missing, latency_ms) = match result.1 {
            Some(latency_ms) => (false, latency_ms),
            None => (true, 0.0),
        };
        s.execute(&[&self.host, &unix_time, &missing, &latency_ms]).unwrap();
    }

    pub fn run(&mut self) {
        loop {
            let ping_start = Instant::now();

            self.record_result(self.do_ping());

            let elapsed = ping_start.elapsed();
            if elapsed < Duration::from_secs(10) {
                sleep(Duration::from_secs(10) - elapsed);
            }
        }
    }
}

struct PageHandler {
    db: DBConn,
}

impl Handler for PageHandler {
    fn handle(&self, r: &mut Request) -> IronResult<Response> {
        let path = &r.url.path;
        let limit = if path.len() > 0 && path[0].len() > 0 {
            if path[0] == "all" {
                None
            } else if let Ok(i) = path[0].parse() {
                Some(i)
            } else {
                return Ok(Response::with((status::NotFound, "404 Not Found")));
            }
        } else {
            Some(600) // default.
        };

        let l = self.db.lock().unwrap();
        let sql = format!("SELECT host, t, missing, latency_ms FROM pings ORDER BY t DESC {}",
                          if let Some(l) = limit {
                              format!("LIMIT {}", l)
                          } else {
                              String::new()
                          });

        let mut s = l.prepare(&sql).unwrap();
        let result = s.query(&[]).unwrap();

        let mut text = String::new();
        text.push_str("<html><body><table \
                       border=\"1\"><tr><th>Hostname</th><th>Time</th><th>Ping</th></tr>");
        for row in result.map(|r| r.unwrap()) {
            let host: String = row.get(0);
            let t: i64 = row.get(1);
            let missing: bool = row.get(2);
            let latency_ms: f64 = row.get(3);
            text.push_str("<tr>");
            let tm = time::at(time::Timespec::new(t, 0));
            text.push_str(&format!("<td>{}</td><td>{}</td>", host, tm.asctime()));
            if missing {
                text.push_str("<td>NO RESPONSE</td>");
            } else {
                text.push_str(&format!("<td>{} ms</td>", latency_ms));
            }
            text.push_str("</tr>");
        }
        text.push_str("</table></body></html>");

        let mut resp = Response::new();
        resp.status = Some(status::Ok);
        resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
        resp.body = Some(Box::new(text));
        Ok(resp)
    }
}

fn main() {
    if env::args().len() < 3 {
        println!("Usage: pingd <sqlite-database> <hostname-to-ping> <http-listen-addr>");
        println!("Example: pingd ping.db myhost localhost:8080");
        return;
    }

    let mut args = env::args().skip(1);
    let dbname = args.next().unwrap();
    let hostname = args.next().unwrap();
    let listen_addr = args.next().unwrap();

    let conn = Arc::new(Mutex::new(rusqlite::Connection::open(&Path::new(&dbname)).unwrap()));
    {
        let l = conn.lock().unwrap();
        let mut s = l.prepare("SELECT name FROM sqlite_master WHERE name = 'pings'").unwrap();
        if s.query(&[]).unwrap().count() == 0 {
            l.prepare("CREATE TABLE pings(host varchar(255), t INTEGER, missing BOOLEAN, \
                       latency_ms FLOAT)")
             .unwrap()
             .execute(&[])
             .unwrap();
        }
    }

    let mut pinger = Pinger::new(hostname, conn.clone());
    let handler = PageHandler { db: conn.clone() };
    let webapp = iron::Iron::new(handler);

    crossbeam::scope(|scope| {
        scope.spawn(|| {
            pinger.run();
        });
        let s: &str = &listen_addr;
        webapp.http(s).unwrap();
    });
}
