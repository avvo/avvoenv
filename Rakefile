require "bundler/setup" rescue nil

task default: [:build, :man]
task build: "target/release/avvoenv"
task man: "avvoenv.1"

task install: [:build, :man] do |t|
  `cp target/release/avvoenv /usr/local/bin`
  `cp avvoenv.1 /usr/local/share/man/man1`
  `which mandb`
  `mandb` if $?.success?
end

file "target/release/avvoenv" => FileList["src/**/*.rs"] do |t|
  `cargo build --release`
end

file "avvoenv.1" => "avvoenv.1.ronn" do |t|
  `ronn --roff #{t.source}`
  File.open(t.name, "r+") do |f|
    original = f.read
    f.rewind
    f << ".ad l\n"
    f << original
  end
end
