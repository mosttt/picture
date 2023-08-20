use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let pixiv = vec![PixivData::default()];
    let mut need_process = Vec::new();

    let v: Vec<_> = pixiv.iter().filter(|y| y.pid == 1).collect();
    need_process.push(v);

    need_process.iter_mut().enumerate().for_each(|(_index, v)| {
        process(v).unwrap();
    });
    //pixiv.push(PixivData::default());

    //println!("pixiv count: {}", pixiv.len());
    println!("need_process count: {}", need_process.len());

    Ok(())
}

//用 'a, 'b 标注或者直接去掉就好了
//why?
fn process<'v, 'p>(_v: &'v mut Vec<&'p PixivData>) -> Result<()>
where
    'p: 'v,
{
    Ok(())
}

#[derive(Default, Debug)]
pub struct PixivData {
    pub pid: u64,
    pub p: u64,
    pub name: String,
}
