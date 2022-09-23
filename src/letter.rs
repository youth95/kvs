pub static LETTER: &str = "小学时 我曾想
如此躺平的我
未来会带着你 上街乞讨
你带着儿 我四处跑
只愿宝 能够温饱

中学时 我曾想
如此深情的我
未来会带着你 天涯海角
你追着风 我守着岛
只愿宝 活的闪耀

大学时 我曾想
如此自强的我
未来会带着你 万人知晓
你思着学 我想着考
只愿宝 无忧无扰

上班时 我曾想
如此温柔的我
未来会带着你 看淡终老
你说着怕 我假装恼
只愿宝 永远年少

爱你 我的宝";

pub fn print_letter() {
  LETTER.split("\n").into_iter().for_each(|line| {
    println!("{}",line);
    std::thread::sleep(std::time::Duration::from_secs(2));
  });
}