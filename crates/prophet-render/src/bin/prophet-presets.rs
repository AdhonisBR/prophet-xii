use prophet_control::ProphetXiiPlugin;
use prophet_params::{ParamId, NUM_PARAMS};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args().nth(1).unwrap_or("presets".into());
    std::fs::create_dir_all(&dir)?;
    let mut p = ProphetXiiPlugin::new(48000.0);
    for i in 0..10 {
        p.load_preset(i);
        let values = (0..NUM_PARAMS)
            .map(|j| {
                p.ipc
                    .params
                    .get(ParamId::from_index(j).unwrap())
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(",");
        let json=format!("{{\"format\":\"PROPHET-XII\",\"version\":1,\"name\":{:?},\"category\":{:?},\"values\":[{}]}}\n",p.presets[i].name,p.presets[i].category.as_str(),values);
        std::fs::write(format!("{dir}/{i:02}-factory.json"), json)?;
    }
    Ok(())
}
