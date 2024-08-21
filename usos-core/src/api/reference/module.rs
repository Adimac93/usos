use std::fmt::Display;

use serde::Deserialize;
use serde_json::Value;

use crate::client::CLIENT;

/// apiref/module
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_module_info(module_name: Module) -> ModuleInfo {
    let response = CLIENT
        .get("https://apps.usos.pwr.edu.pl/services/apiref/module")
        .query(&[("name", format!("services/{module_name}"))])
        .send()
        .await
        .unwrap();

    let json = response.json::<ModuleInfo>().await.unwrap();
    json
}

#[tokio::test]
#[ignore]
async fn test_get_module_info() {
    let module_info = get_module_info(Module::ApiReference).await;
    println!("{:?}", module_info);
}

pub enum Module {
    ApiReference,
    ApiServerData,
    ApiStats,
    Attendance,
    Blobbox,
    Calendar,
    Cards,
    Courses,
    Credits,
    CourseTests,
    CustomGroups,
    StudentRecordsTransfer,
    EventSubsription,
    EventsEdition,
    ExamReports,
    ExamReporstExtra,
    Exams,
    Faculties,
    FacultyPermissions,
    FeedbackReports,
    FileShare,
    GeographicalData,
    Grades,
    Groups,
    Guide,
    Housing,
    InstitutionalAdresses,
    MailClient,
    Mailing,
    Meetings,
    Mobility,
    News,
    OAuth,
    OAuth2,
    Payments,
    Phones,
    Photos,
    Pit,
    PlacementTests,
    PrimaryGroups,
    StudyPrograms,
    Registrations,
    ClearanceSlips,
    Statements,
    Surveys,
    Terms,
    Theses,
    TimeTables,
    UserPreferences,
    Users,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Module::ApiReference => "apiref",
                Module::ApiServerData => "apisrv",
                Module::ApiStats => "apistlog",
                Module::Attendance => "attendance",
                Module::Blobbox => "blobbox",
                Module::Calendar => "calendar",
                Module::Cards => "cards",
                Module::Courses => "courses",
                Module::Credits => "credits",
                Module::CourseTests => "crstests",
                Module::CustomGroups => "csgroups",
                Module::StudentRecordsTransfer => "emrex",
                Module::EventSubsription => "events",
                Module::EventsEdition => "events2",
                Module::ExamReports => "examrep",
                Module::ExamReporstExtra => "examrep2",
                Module::Exams => "exams",
                Module::Faculties => "fac",
                Module::FacultyPermissions => "facperms",
                Module::FeedbackReports => "feedback",
                Module::FileShare => "fileshare",
                Module::GeographicalData => "geo",
                Module::Grades => "grades",
                Module::Groups => "groups",
                Module::Guide => "guide",
                Module::Housing => "housing",
                Module::InstitutionalAdresses => "instaddr",
                Module::MailClient => "mailclient",
                Module::Mailing => "mailing",
                Module::Meetings => "meetings",
                Module::Mobility => "mobility",
                Module::News => "news",
                Module::OAuth => "oauth",
                Module::OAuth2 => "oauth2",
                Module::Payments => "payments",
                Module::Phones => "phones",
                Module::Photos => "photos",
                Module::Pit => "pit",
                Module::PlacementTests => "plctests",
                Module::PrimaryGroups => "prgroups",
                Module::StudyPrograms => "progs",
                Module::Registrations => "registrations",
                Module::ClearanceSlips => "slips",
                Module::Statements => "statements",
                Module::Surveys => "surveys",
                Module::Terms => "terms",
                Module::Theses => "theses",
                Module::TimeTables => "tt",
                Module::UserPreferences => "uprefs",
                Module::Users => "users",
            }
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ModuleInfo {
    name: String,
    title: String,
    brief_description: String,
    description: String,
    submodules: Vec<String>,
    methods: Vec<String>,
    beta: bool,
}
