use itertools::Itertools;

pub async fn clone(remote_url: String, _target_directory: String) -> anyhow::Result<()> {
    let remote_url = format!("{remote_url}.git");

    let mut url_with_params = reqwest::Url::parse(&remote_url)?;

    url_with_params.set_query(Some("service=git-upload-pack"));

    {
        let mut p = url_with_params.path_segments_mut().unwrap();
        p.push("info");
        p.push("refs");
    }

    let response_raw = reqwest::get(url_with_params.clone()).await?.text().await?;
    let response_lines = response_raw.split('\n').collect_vec();

    for line in &response_lines {
        println!("{:?}", &line);
    }

    Ok(())
}
