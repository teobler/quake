use std::fs;
use std::path::PathBuf;

use quake_microsoft_todo::tasks::{TodoTask, WellknownListName};

use crate::sqlite_to_file::simple_escape;
use quake_core::entry::entry_file::EntryFile;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputList {
    pub display_name: String,
    pub id: String,
    pub wellknown_list_name: WellknownListName,
    pub children: Vec<TodoTask>,
}

pub fn dump_microsoft_todo(todos_lists: Vec<OutputList>, path: &PathBuf) {
    let _ = fs::create_dir(&path);
    let mut index = 1;
    for list in todos_lists {
        for todo in list.children {
            let mut file = EntryFile::default();

            let title = todo.title;
            file.add_field("category", format!("{:?}", list.display_name));
            file.add_field("title", format!("{:?}", simple_escape(title.clone())));
            file.add_field("created_date", format!("{:?}", todo.created_date_time));
            file.add_field(
                "updated_date",
                format!("{:?}", todo.last_modified_date_time),
            );

            let time = match todo.reminder_date_time {
                None => "".to_string(),
                Some(dat) => dat.date_time,
            };
            file.add_field("reminder_date", format!("{:?}", time));

            let completed_date = match todo.completed_date_time {
                None => "".to_string(),
                Some(dat) => dat.date_time,
            };
            file.add_field("completed_date", format!("{:?}", completed_date));

            let due_date = match todo.due_date_time {
                None => "".to_string(),
                Some(dat) => dat.date_time,
            };
            file.add_field("due_date", format!("{:?}", due_date));

            file.add_field("importance", format!("{:?}", todo.importance));
            file.add_field("status", format!("{:?}", todo.status));

            file.name = EntryFile::file_name(index, title.as_str());

            file.content = "\n\n".to_string();
            file.content.push_str(todo.body.content.as_str());

            match fs::write(path.join(file.name.clone()), file.to_string()) {
                Ok(_) => {}
                Err(err) => {
                    println!("{:?}", file.name.clone());
                    println!("{:?}", err);
                }
            }

            index = index + 1
        }
    }
}
