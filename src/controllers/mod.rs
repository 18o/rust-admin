use std::collections::HashMap;
use std::fmt::Debug;
use fluffy::{ tmpl::Tpl, response, model::Model, model::Db, data_set::DataSet, db, };
use crate::models::ModelBackend;
use actix_web::{HttpResponse, web::{Path, Form}};
use crate::caches;
use serde::ser::{Serialize};

pub trait Controller { 
    
    /// 模型
    type M: ModelBackend + Default + Serialize + Debug;
    
    /// 得到控制器名称
    fn get_controller_name() -> &'static str { 
        Self::M::get_table_name()
    }
    
    /// 主頁
    fn index(tpl: Tpl) -> HttpResponse { 
        let controller_name = Self::get_controller_name(); //控制器名称
        let info = Self::M::get_records();
        let breads = caches::menus::BREADS.lock().unwrap();
        let bread_path = if let Some(v) = breads.get(&format!("/{}", controller_name)) { v } else { "" };
        let data = tmpl_data![
            "action_name" => &"index",
            "controller_name" => &controller_name,
            "records" => &info.records,
            "pager" => &info.pager,
            "bread_path" => &bread_path,
        ];
        let view_file = &format!("{}/index.html", controller_name);
        render!(tpl, view_file, &data)
    }

    /// 处理编辑时需要展现出来的附加数据
    fn edit_after(_data: &mut tera::Context) {}

    /// 編輯
    fn edit(info: Path<usize>, tpl: Tpl) -> HttpResponse { 
        let controller_name = Self::get_controller_name(); //控制器名称
        let id = info.into_inner();
        let is_update = id > 0;
        let row = if !is_update { Self::M::get_default() } else { 
            let fields = Self::M::get_fields();
            let query = query![fields => &fields, ];
            let cond = cond!["id" => id,];
            let mut conn = fluffy::db::get_conn();
            if let Some(r) = Self::M::fetch_row(&mut conn, &query, Some(&cond)) { 
                Self::M::get_record(r)
            } else { Self::M::get_default() }
        };
        let button_text = if is_update { "保存记录" } else { "添加记录" };
        let mut data = tmpl_data![
            "controller_name" => controller_name,
            "row" => &row,
            "button_text" => button_text,
            "id" => &id,
        ];
        Self::edit_after(&mut data);
        let view_file = &format!("{}/edit.html", controller_name);
        render!(tpl, view_file, &data)
    }

    /// 編輯
    fn save(info: Path<usize>, post: Form<HashMap<String, String>>) -> HttpResponse { 
        let id = info.into_inner();
        if id == 0 { Self::save_for_create(post) } else { Self::save_for_update(id, post) }
    }

    /// 添加
    fn save_for_create(post: Form<HashMap<String, String>>) -> HttpResponse { 
        let post_fields = post.into_inner();
        if let Err(message) = Self::M::validate(&post_fields) {  //如果检验出错
            return response::error(message);
        }
        let table_name = Self::M::get_table_name();
        let table_fields = caches::TABLE_FIELDS.lock().unwrap();
        let mut checked_fields = Db::check_fields(table_name, &table_fields, post_fields, false); //經過檢驗之後的數據
        Self::M::save_before(&mut checked_fields); //对于保存数据前的检测
        let mut data = DataSet::create();
        for (k, v) in &checked_fields { 
            data.set(k, v);
        }
        let mut conn = db::get_conn();
        let id = Self::M::create(&mut conn, &data);
        if id > 0 { 
            return response::ok();
        } 
        response::error("增加記錄失敗")
    }
    
    /// 修改
    fn save_for_update(id: usize, post: Form<HashMap<String, String>>) -> HttpResponse { 
        let post_fields = post.into_inner();
        if let Err(message) = Self::M::validate(&post_fields) {  //如果检验出错
            return response::error(message);
        }
        let table_name = Self::M::get_table_name();
        let table_fields = caches::TABLE_FIELDS.lock().unwrap();
        let mut checked_fields = Db::check_fields(table_name, &table_fields, post_fields, true); //經過檢驗之後的數據
        Self::M::save_before(&mut checked_fields); //对于保存数据前的检测
        let mut data = DataSet::update();
        for (k, v) in &checked_fields { 
            data.set(k, v);
        }
        let mut conn = db::get_conn();
        let cond = cond![ "id" => &id, ];
        let id = Self::M::update(&mut conn, &data, &cond);
        if id > 0 { 
            return response::ok();
        } 
        response::error("修改記錄失敗")
    }
    
    /// 刪除
    fn delete(id_strings: Path<String>) -> HttpResponse { 
        let mut ids_string = String::new();
        for (index, value) in id_strings.split(",").enumerate() { 
            let _ = if let Ok(v) = value.parse::<usize>() { v } else { return response::error("错误的参数"); };
            if index > 0 { 
                ids_string.push_str(",");
            }
            ids_string.push_str(value);
        }
        let cond = cond![
            in_range => ["id" => &ids_string,],
        ];
        let mut conn = db::get_conn();
        let affected_rows = Self::M::delete(&mut conn, &cond);
        if affected_rows == 0 { response::error("未删除任何记录") } else { response::ok() }
    }
}

pub mod index;
pub mod admins;
pub mod admin_roles;
pub mod menus;
pub mod users;
pub mod video_categories;
pub mod videos;
pub mod video_replies;
pub mod video_tags;
pub mod user_levels;
pub mod watch_records;
pub mod ads;
