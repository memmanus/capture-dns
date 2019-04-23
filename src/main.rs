use failure::{Error, format_err};
use structopt::StructOpt;
use trust_dns_proto::op::message::Message;
use trust_dns_proto::serialize::binary::BinDecodable;
use trust_dns_proto::rr::record_data::RData;

#[derive(StructOpt)]
#[structopt(name = "capture-dns", about = "Capture DNS requests and show their QNames")]
struct Opt {
  #[structopt(help = "device", default_value = "wlan0")]
  device: String,
}

fn process(packet: &[u8]) {
  // 42 = 14 Ethernet header + 20 IPv4 header + 8 UDP header
  match Message::from_bytes(&packet[42..]) {
    Ok(msg) => {
      let qname;
      let name;
      match msg.queries().iter().nth(0) {
        Some(q) => {
          qname = q.name().to_string();
          name = qname.trim_end_matches('.');
        },
        None => { return },
      };

      let mut is_first = true;
      for a in msg.answers() {
        let arrow = if is_first { "=>" } else { "->" };

        match a.rdata() {
          RData::A(ip) => {
            println!("{} {} {}", name, arrow, ip);
          },
          RData::AAAA(ip) => {
            println!("{} {} {}", name, arrow, ip);
          },
          RData::CNAME(cname) => {
            let s = cname.to_string();
            let name = s.trim_end_matches('.');
            println!("{} {} {}", name, arrow, name);
          },
          _ => { },
        }

        is_first = false;
      }
    },
    Err(e) => eprintln!("Error: {:?}", e),
  }
}

fn main() -> Result<(), Error> {
  let opt = Opt::from_args();
  let device = pcap::Device::list()?.into_iter()
    .filter(|d| d.name == opt.device).nth(0)
    .ok_or_else(|| format_err!("device {} not found", opt.device))?;

  let mut cap = device.open()?;
  cap.filter("ip proto \\udp and src port 53")?;
  loop {
   let packet = cap.next()?;
   process(&packet);
  }
}
