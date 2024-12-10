use anyhow::Context;
use bytes::Bytes;
use reqwest::Client;
use serde_json::{json, Value};

// let likes = get_likes("username").await?;
//
// for post in &likes.posts {
//     let embeds = get_post_image_embeds(post).await?;
//
//     for image_ref in &embeds.refs {
//         let bytes = load_image(&embeds.did, image_ref).await?;
//         let path = format!("images/{image_ref}.jpg");
//         File::create(path).await?.write_all(&bytes).await?;
//     }
//
//     // for image in embeds.i
// }

pub async fn get_like_image_urls(handle: &str) -> anyhow::Result<Vec<String>> {
    let likes = get_likes(handle).await?;
    let mut urls = vec![];
    for post in &likes.posts {
        let Ok(embeds) = get_post_image_embeds(post).await else {
            continue;
        };

        for image_ref in embeds.refs {
            let url = format!(
                "https://bsky.social/xrpc/com.atproto.sync.getBlob?did={did}&cid={cid}",
                did = embeds.did,
                cid = image_ref
            );
            urls.push(url);
        }
    }

    Ok(urls)
}

async fn load_image(did: &str, image_ref: &str) -> anyhow::Result<Bytes> {
    let request = Client::new()
        .get("https://bsky.social/xrpc/com.atproto.sync.getBlob")
        .query(&json! {{
            "did": did,
            "cid": image_ref,
        }});

    Ok(request.send().await?.bytes().await?)
}

#[derive(Debug)]
struct LikedPost {
    repo: String,
    collection: String,
    rkey: String,
    cid: String,
}

impl LikedPost {
    fn from_value(value: &Value) -> Option<Self> {
        let subject = value.get("value")?.get("subject")?;
        let mut uri = subject.get("uri")?.as_str()?.split('/');

        Some(Self {
            repo: uri.nth(2)?.into(),
            collection: uri.next()?.into(),
            rkey: uri.next()?.into(),
            cid: subject.get("cid")?.as_str()?.into(),
        })
    }
}

#[derive(Debug)]
struct Likes {
    posts: Vec<LikedPost>,
    cursor: String,
}

async fn get_likes(handle: &str) -> anyhow::Result<Likes> {
    let request = Client::new()
        .get("https://bsky.social/xrpc/com.atproto.repo.listRecords")
        .query(&json! {{
            "repo": handle,
            "collection": "app.bsky.feed.like",
            "limit": 10,
            // "cursor": "string",
        }});
    let json = request.send().await?.json::<Value>().await?;

    let records = json
        .get("records")
        .and_then(Value::as_array)
        .context("no records")?;
    let posts = records.iter().filter_map(LikedPost::from_value).collect();
    let cursor = json
        .get("cursor")
        .and_then(Value::as_str)
        .context("no cursor")?
        .into();

    Ok(Likes { posts, cursor })
}

async fn get_profile(handle: &str) -> anyhow::Result<String> {
    let request = Client::new()
        .get("https://bsky.social/xrpc/com.atproto.repo.getRecord")
        .query(&json! {{
            "repo": handle,
            "collection": "app.bsky.actor.profile",
            "rkey": "self",
        }});
    Ok(request.send().await?.text().await?)
}

#[derive(Debug)]
struct PostImageEmbeds {
    did: String,
    refs: Vec<String>,
}

async fn get_post_image_embeds(post: &LikedPost) -> anyhow::Result<PostImageEmbeds> {
    let request = Client::new()
        .get("https://bsky.social/xrpc/com.atproto.repo.getRecord")
        .query(&json! {{
            "repo": post.repo,
            "collection": post.collection,
            "rkey": post.rkey,
            "cid": post.cid,
        }});
    let result = request.send().await?.json::<Value>().await?;

    let did = result.get("uri").and_then(Value::as_str);
    let did = did.and_then(|d| d.split('/').nth(2)).context("no did")?;

    let refs = result
        .get("value")
        .and_then(|v| v.get("embed"))
        .and_then(|v| v.get("images"))
        .and_then(Value::as_array)
        .map(|v| v.iter())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("image"))
        .filter_map(|v| v.get("ref"))
        .filter_map(|v| v.get("$link"))
        .filter_map(|v| v.as_str())
        .map(|v| v.into())
        .collect();

    Ok(PostImageEmbeds {
        did: did.into(),
        refs,
    })
}
