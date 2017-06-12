require "rake/clean"
CLEAN.include("gem")
CLOBBER.include("target", "avvoenv.1")

task default: [:"build:default", :man]
namespace :build do
  task default: "target/release/avvoenv"
  task linux: "target/x86_64-unknown-linux-musl/release/avvoenv"
end
task man: "avvoenv.1"

task install: [:"build:default", :man] do |t|
  prefix = ENV["PREFIX"] || "."
  `mkdir -p #{prefix}/bin`
  `mkdir -p #{prefix}/share/man/man1`
  `cp target/release/avvoenv #{prefix}/bin`
  `cp avvoenv.1 #{prefix}/share/man/man1`
  `which mandb`
  `mandb` if $?.success?
end

file "target/release/avvoenv" => FileList["src/**/*.rs"] do |t|
  `cargo build --release`
end

file "target/x86_64-unknown-linux-musl/release/avvoenv" => FileList["src/**/*.rs"]  do |t|
  `docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release`
end

file "avvoenv.1" => "avvoenv.1.ronn" do |t|
  `gem install -g -i gem -n gem/bin`
  ENV["GEM_PATH"] = "./gem"
  `gem/bin/ronn --roff #{t.source}`
  File.open(t.name, "r+") do |f|
    original = f.read
    f.rewind
    f << ".ad l\n"
    f << original
  end
end
