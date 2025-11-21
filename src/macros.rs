#[macro_export]
macro_rules! string {
    ($x:expr) => {
        String::from($x)
    };
}
#[cfg(feature = "ssr")]
#[macro_export]
macro_rules! record_id {
    ($table:expr, $id:expr) => {
        surrealdb::RecordId::from(($table,$id))
    };
}
#[cfg(feature = "ssr")]
#[macro_export]
macro_rules! exercise_db {
    ($exercise:expr) => {
        crate::backend::infrastructure::db::ExerciseDB::from($exercise)
    };
}


#[cfg(feature = "ssr")]
#[macro_export]
macro_rules! day_db {
    ($day:expr) => {
        crate::backend::infrastructure::db::DayDB::from($day)
    };
}