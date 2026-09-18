use crate::Error;
use tauri::{Manager, Runtime};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum InputType
{
    Number,
    Time,
    Text
}
#[derive(Serialize, Deserialize)]
pub struct Entry
{
pub suggested_amount:u32,
pub args: Vec<Arg>
}

#[derive(Serialize, Deserialize)]
pub struct Arg
{
pub name: String,
pub input: InputType
}

#[derive(Serialize, Deserialize)]
pub struct Header
{
pub title: String,
pub args: Vec<Arg>
}

#[derive(Serialize, Deserialize)]
pub struct Section
{
pub header: Header, 
pub blocks: Entry
}

#[derive(Serialize, Deserialize)]
pub struct Plan 
{
pub title: String,
pub cards: Vec<Section>
}




impl Plan {
    fn validate(&self) -> Result<(), Error> {
        if self.title.trim().is_empty(){  
            return Err(Error{message: "Empty Title Field".to_string()});
        };
        if self.cards.is_empty(){
            return Err(Error{message: "Empty Cards".to_string()});
        };
        if self.cards.iter().any(
            |card| card.header.title.is_empty()
        ) {
            return Err(Error { message: "Empty cards Header or Empty Block Args".to_string()});
        }

        Ok(())
    }
    
    fn deserialize_template(template_json: String) -> Result<Self, Error> {
    let template: Self= serde_json::from_str(&template_json).map_err(|e| Error{message: e.to_string()})?;
    Ok(template)
    }
    
    fn serialize_template(&self) -> Result<String, Error> {
    let template_json: String = serde_json::to_string(&self).map_err(|e| Error{message: e.to_string()})?;
    Ok(template_json)
}

    fn write_file<R: Runtime>(&self, app: tauri::AppHandle<R>, file_name: &str) -> Result<(), Error> {
        self.validate()?;
        let json_data = self.serialize_template()?;
        let mut path = app.path().app_data_dir()
            .map_err(|e| Error{message: e.to_string()})?;

        std::fs::create_dir_all(&path)
            .map_err(|e| Error { message: e.to_string() })?;

        path.push(file_name);
        std::fs::write(path, json_data)
            .map_err(|e| Error{message: e.to_string()})?;

        Ok(())
    }

    fn read_file<R: Runtime>(app: tauri::AppHandle<R>, file_name: &str) -> Result<Self, Error> {
        let mut path = app.path().app_data_dir()
            .map_err(|e| Error{message: e.to_string()})?;

        path.push(file_name);

        let json_data = std::fs::read_to_string(path)
            .map_err(|e| Error{message: e.to_string()})?;

        let template = Self::deserialize_template(json_data)?;
        template.validate()?;

        Ok(template)
    }
}

#[tauri::command]
async fn write_template<R: Runtime>(app: tauri::AppHandle<R>, json: String, file_name: String) -> Result<(), Error> {
  let template = Plan::deserialize_template(json)?;
  template.write_file(app, &file_name)?;
  Ok(())
}

#[tauri::command]
async fn read_template<R: Runtime>(app: tauri::AppHandle<R>, file_name: String) -> Result<Plan, Error> {
  let template:Plan = Plan::read_file(app, &file_name)?;
  Ok(template)
}