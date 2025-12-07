use crate::models::todo::Todo;

pub async fn get_todo(id: u32) -> Result<Todo, reqwest::Error> {
    let url = format!("https://jsonplaceholder.typicode.com/todos/{}", id);

    let res = reqwest::get(&url).await?.json::<Todo>().await?;

    Ok(res)
}
