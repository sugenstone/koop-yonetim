mod db;
mod commands;
mod kasa_commands;
mod hissedar_commands;
mod donem_commands;
mod hisse_commands;
mod aidat_commands;
mod gelir_gider_commands;

pub use commands::*;
pub use kasa_commands::*;
pub use hissedar_commands::*;
pub use donem_commands::*;
pub use hisse_commands::*;
pub use aidat_commands::*;
pub use gelir_gider_commands::*;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // SQLite veritabanını başlat
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Uygulama veri dizini bulunamadı");
            std::fs::create_dir_all(&app_data_dir).expect("Veri dizini oluşturulamadı");
            let db_path = app_data_dir.join("flowbite_admin.db");
            db::init_db(&db_path.to_string_lossy())
                .expect("Veritabanı başlatılamadı");
            app.manage(db::DbState::new(&db_path.to_string_lossy())
                .expect("DbState oluşturulamadı"));
            Ok(())
        })
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_users,
            commands::create_user,
            commands::update_user,
            commands::delete_user,
            commands::get_products,
            commands::create_product,
            commands::update_product,
            commands::delete_product,
            commands::get_dashboard_stats,
            // Kasa komutları
            kasa_commands::get_kasalar,
            kasa_commands::get_kasa,
            kasa_commands::create_kasa,
            kasa_commands::update_kasa,
            kasa_commands::delete_kasa,
            kasa_commands::get_kasa_hareketleri,
            kasa_commands::create_kasa_hareketi,
            kasa_commands::delete_kasa_hareketi,
            // Kasa transfer komutları
            kasa_commands::kasa_transfer,
            kasa_commands::get_kasa_transferleri,
            // Hisse komutları
            hisse_commands::get_hisseler,
            hisse_commands::get_hisse,
            hisse_commands::create_hisse,
            hisse_commands::create_hisse_toplu,
            hisse_commands::delete_hisse,
            hisse_commands::get_hisse_atamalari,
            hisse_commands::get_hissedar_atamalari,
            hisse_commands::hisse_ata,
            hisse_commands::hisse_atama_sil,
            hisse_commands::hisse_transfer,
            hisse_commands::hisse_satis_baslat,
            hisse_commands::hisse_satis_odeme_ekle,
            hisse_commands::get_hisse_satis_aktif,
            hisse_commands::get_hisse_satis_odemeleri,
            hisse_commands::hisse_satis_iptal,
            // Hissedar komutları
            hissedar_commands::get_hissedarlar,
            hissedar_commands::get_hissedar,
            hissedar_commands::create_hissedar,
            hissedar_commands::update_hissedar,
            hissedar_commands::delete_hissedar,
            // Dönem komutları
            donem_commands::get_donemler,
            donem_commands::get_donem,
            donem_commands::create_donem,
            donem_commands::update_donem,
            donem_commands::delete_donem,
            // Toplantı komutları
            donem_commands::get_toplantilar,
            donem_commands::create_toplanti,
            donem_commands::update_toplanti,
            donem_commands::delete_toplanti,
            // Karar komutları
            donem_commands::get_kararlar,
            donem_commands::create_karar,
            donem_commands::update_karar,
            donem_commands::delete_karar,
            // Aidat borç komutları
            aidat_commands::donem_borc_olustur,
            aidat_commands::get_donem_borclari,
            aidat_commands::get_hisse_borclari,
            aidat_commands::get_hissedar_borclari,
            // Cüzdan komutları
            aidat_commands::get_hissedar_cuzdani,
            aidat_commands::hissedar_cuzdan_para_ekle,
            // Gelir/Gider komutları
            gelir_gider_commands::get_gelir_gider_kategorileri,
            gelir_gider_commands::create_gelir_gider_kategori,
            gelir_gider_commands::update_gelir_gider_kategori,
            gelir_gider_commands::delete_gelir_gider_kategori,
            gelir_gider_commands::get_gelir_gider_kayitlari,
            gelir_gider_commands::create_gelir_gider_kaydi,
            gelir_gider_commands::delete_gelir_gider_kaydi,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri uygulaması çalıştırılamadı");
}
