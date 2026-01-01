use crate::config::Config;
use crate::models::youtube_transcript::Root as TranscriptRoot;
use crate::services::supabase_service::get_youtube_transcript;
use crate::{
    models::youtube_snippet::SearchResult, services::youtube_service::get_detail_byLink,
    services::youtube_service::get_youtube_search,
};
use tokio::fs;

// Function to get the latest KS Forward video, process its transcript, chat with AI, and send to Discord
pub async fn get_lastest_ksForword(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let ks_channel_id = &config.ksforword_channel_id;

    let resYoutube = get_youtube_search(&ks_channel_id).await?;
    let filtered: Vec<_> = resYoutube
        .items
        .iter()
        .filter(|item| {
            if let Some(title) = &item.snippet.title {
                title.starts_with("KS Forward")
            } else {
                false
            }
        })
        .collect();

    let lastest = filtered.first();

    if let Some(item) = lastest {
        let video_id = item
            .id
            .as_video_id()
            .unwrap_or_else(|| "No video id found".to_string());

        println!("video id: {}", video_id);

        let mapped = SearchResult {
            video_id: item.id.as_video_id().unwrap_or_default(),
            link: format!("https://www.youtube.com/watch?v={}", video_id),
            title: item.snippet.title.clone().unwrap_or_default(),
            publish_time: item.snippet.publish_time.clone().unwrap_or_default(),
        };

        println!("Found KS Forward Video: {}", mapped.title);

        // Get mock transcript and parse
        let use_mock_data = config.use_mock_data;
        let transcript_json = if use_mock_data {
            dummy_transcript().await?
        } else {
            get_youtube_transcript(&mapped.link).await?
        };
        print!("Transcript fetched.");

        let full_transcript = parse_transcript_fullscript(transcript_json).await?;
        println!("Full Transcript length: {}", full_transcript.len());

        if full_transcript != "" && full_transcript.len() > 0 {
            println!("Transcript successfully retrieved and parsed.");
            
            //chat with AI
            let ai_response =
                crate::services::myAI_service::chat_with_ai(config, full_transcript).await?;
            let ai_answer = ai_response.answer;
            println!("AI Answer length: {}", ai_answer.len());

            // send to discord
            let message = ai_answer;
            crate::services::discord_service::send_message(&mapped.title, &message).await?;
            println!("Message sent to Discord.");
            println!("KS Forward processing completed.");
        } else {
            println!("Transcript is empty.");
        }
    } else {
        println!("No found data :  KS Forward");
    }

    Ok(())
}

// Function to get summary link from video link
pub async fn get_summary_link(
    config: &Config,
    video_link: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let detail = get_detail_byLink(video_link).await?;
    if detail.items.is_empty() {
        return Err("No video details found for the provided link".into());
    }

    print!(
        "Video Title: {}",
        detail.items[0].snippet.title.clone().unwrap_or_default()
    );

    let transcript_json = get_youtube_transcript(video_link).await?;
    print!("Transcript JSON fetched.");

    let full_transcript = parse_transcript_fullscript(transcript_json).await?;
    print!("Full transcript parsed.");
    print!("Transcript length: {}", full_transcript.len());

    let ai_response = crate::services::myAI_service::chat_with_ai(config, full_transcript).await?;
    let ai_answer = ai_response.answer;
    //print!("AI Answer: {}", ai_answer);

    //send to discord
    let message = ai_answer.clone();
    crate::services::discord_service::send_message(
        &detail.items[0].snippet.title.clone().unwrap_or_default(),
        &message,
    )
    .await?;

    Ok(ai_answer)
}

// Function to parse transcript JSON into full transcript string
pub async fn parse_transcript_fullscript(
    transcript_json: TranscriptRoot,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = transcript_json.content;
    let mut full_transcript = String::new();
    for entry in content {
        full_transcript.push_str(&entry.text);
        full_transcript.push(' '); // เพิ่มช่องว่างระหว่างข้อความ
    }

    Ok(full_transcript)
}

// Mock function for testing transcript parsing
pub async fn dummy_transcript() -> Result<TranscriptRoot, Box<dyn std::error::Error>> {
    let path = "src/mock_data/example_transcript.json";
    let data = fs::read_to_string(path).await?;
    let transcript: TranscriptRoot = serde_json::from_str(&data)?;
    Ok(transcript)
}
