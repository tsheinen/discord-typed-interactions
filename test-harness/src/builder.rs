mod included {
    include!(concat!(env!("OUT_DIR"), "/ctf_gen.rs"));
}
use included::ctf;
use ctf::Options;

fn main() {

}