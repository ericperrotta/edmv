default:
  rm -rf tmp
  mkdir tmp
  touch tmp/{foo,bar,baz}
  cargo run tmp/*
  ls tmp
