//! Scopes present in the USOS API.

use std::{collections::HashSet, fmt::Display, str::FromStr};

use serde::Deserialize;

/// A wrapper struct that contains a set of authorization scopes.
#[derive(Debug, Clone)]
pub struct Scopes(HashSet<Scope>);

impl Scopes {
    pub fn new(scopes: HashSet<Scope>) -> Self {
        Self(scopes)
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|scope| scope.to_string())
                .collect::<Vec<_>>()
                .join("|")
        )
    }
}

#[derive(Deserialize, Debug, Hash, Eq, PartialEq, Clone, Copy)]
/// /services/apiref/scopes
pub enum Scope {
    /// Allows access to get administration documents etc.
    #[serde(rename = "adm_documents")]
    AdministrativeDocuments,
    /// Provides access to user's ID cards data, such as chip uid or expiration date
    #[serde(rename = "cards")]
    Cards,
    /// Allows you to change user preferences (via the uprefs module). You may need some other scopes in order to change or view some of the preferences. Also, the access to some important preferences may be restricted in other ways, i.e. only Administrative Consumers may be allowed to change them.
    #[serde(rename = "change_all_preferences")]
    ChangeAllPreferences,
    /// Provides access to details and results of user's course tests.
    #[serde(rename = "crstests")]
    CourseTests,
    /// Provides access to administrative housing operations on user's behalf. For more information, please visit the housing module.
    #[serde(rename = "dorm_admin")]
    Dorms,
    /// Allows editing user's attributes (the same thet the user can edit on his USOSweb profile page).
    #[serde(rename = "edit_user_attrs")]
    ChangeUserAttributes,
    /// Provides access to user's email address.
    #[serde(rename = "email")]
    Email,
    /// Allows access to user's preferences, push notifications, etc.
    #[serde(rename = "events")]
    Events,
    /// Provides access to grades information.
    #[serde(rename = "grades")]
    Grades,
    /// Allows access to read and write exam reports.
    #[serde(rename = "grades_write")]
    GradesWrite,
    /// Provides access to the mailclient module (in the name of your user). Currently only a small set of methods is available for non-administrative consumers, but this set will be growing.
    #[serde(rename = "mailclient")]
    MailClient,
    /// Provides access to user's personal mobile phone number(s).
    #[serde(rename = "mobile_numbers")]
    MobileNumbers,
    /// Enables your application to perform authorized requests on behalf of the user at any time. By default, Access Tokens expire after a short time period to ensure applications only make requests on behalf of users when they are actively using the application. This scope makes Access Tokens long-lived.
    #[serde(rename = "offline_access")]
    OfflineAccess,
    /// Provides access to email addresses of other users (i.e. the ones related to your user).
    #[serde(rename = "other_emails")]
    OtherEmails,
    /// Allows access to your payments.
    #[serde(rename = "payments")]
    Payments,
    /// Provides access to user's personal data, such as PESEL number, date of birth, etc.
    #[serde(rename = "personal")]
    Personal,
    /// Provides read access to user's photo and his/her photo visibility preferences ("who can see my photo?").
    #[serde(rename = "photo")]
    Photo,
    /// Provides access to results of user's placement tests in foreign languages.
    #[serde(rename = "placement_tests")]
    PlacementTests,
    /// Allows access to official permissions related to the user's session debugging rights. Allows you to get the answer to the question "Is my user permitted to debug the session of user X?". See "can_i_debug" field of the services/users/user method for more information.
    #[serde(rename = "session_debugging_perms")]
    SessionDebugging, // dev only
    /// Provides access to most of the actions within the Clearance Slips module. With this scope you can view, create and edit slips, answer questions and perform any non-administrative action which the user can perform via USOSweb. You will need an additional 'slips_admin' scope if you want to manage slip templates too.
    #[serde(rename = "slips")]
    ClearanceSlips,
    /// Provides access to template management of the "slips" module. That is, it allows you to create and edit questions, mark templates as obsolete etc.
    #[serde(rename = "slips_admin")]
    ClearanceSlipsAdmin,
    /// If your user is a staff member, then this scope provides access to some common student-related data usually visible only to staff members, e.g. student numbers, or broader lists of students' study programmes.
    #[serde(rename = "staff_perspective")]
    StaffPerspective,
    /// Provides access to lists of student's exams, information on their examiners, places the exams take place etc.
    #[serde(rename = "student_exams")]
    StudentExams,
    /// Allows to register and unregister the student from his exams.
    #[serde(rename = "student_exams_write")]
    StudentExamsEdit,
    /// Provides access to lists of programmes, courses, classes and groups which the user attends (as a student).
    #[serde(rename = "studies")]
    Studies,
    /// Allows access to surveys from students point of view.  With this scope you can fetch and fill out surveys.
    #[serde(rename = "surveys_filling")]
    SurveysFilling,
    /// Allows access to reports on surveys that concern user as a lecturer.
    #[serde(rename = "surveys_reports")]
    SurveysReports,
    /// Allows access to editing diploma exam protocols, e.g. signing protocols.
    #[serde(rename = "theses_protocols_write")]
    ThesesProtocolsEdit,
}

impl FromStr for Scope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "adm_documents" => Ok(Self::AdministrativeDocuments),
            "cards" => Ok(Self::Cards),
            "change_all_preferences" => Ok(Self::ChangeAllPreferences),
            "crstests" => Ok(Self::CourseTests),
            "dorm_admin" => Ok(Self::Dorms),
            "edit_user_attrs" => Ok(Self::ChangeUserAttributes),
            "email" => Ok(Self::Email),
            "events" => Ok(Self::Events),
            "grades" => Ok(Self::Grades),
            "grades_write" => Ok(Self::GradesWrite),
            "mailclient" => Ok(Self::MailClient),
            "mobile_numbers" => Ok(Self::MobileNumbers),
            "offline_access" => Ok(Self::OfflineAccess),
            "other_emails" => Ok(Self::OtherEmails),
            "payments" => Ok(Self::Payments),
            "personal" => Ok(Self::Personal),
            "photo" => Ok(Self::Photo),
            "placement_tests" => Ok(Self::PlacementTests),
            "session_debugging_perms" => Ok(Self::SessionDebugging),
            "slips" => Ok(Self::ClearanceSlips),
            "slips_admin" => Ok(Self::ClearanceSlipsAdmin),
            "staff_perspective" => Ok(Self::StaffPerspective),
            "student_exams" => Ok(Self::StudentExams),
            "student_exams_write" => Ok(Self::StudentExamsEdit),
            "studies" => Ok(Self::Studies),
            "surveys_filling" => Ok(Self::SurveysFilling),
            "surveys_reports" => Ok(Self::SurveysReports),
            "theses_protocols_write" => Ok(Self::ThesesProtocolsEdit),
            _ => Err(()),
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdministrativeDocuments => write!(f, "adm_documents"),
            Self::Cards => write!(f, "cards"),
            Self::ChangeAllPreferences => write!(f, "change_all_preferences"),
            Self::CourseTests => write!(f, "crstests"),
            Self::Dorms => write!(f, "dorm_admin"),
            Self::ChangeUserAttributes => write!(f, "edit_user_attrs"),
            Self::Email => write!(f, "email"),
            Self::Events => write!(f, "events"),
            Self::Grades => write!(f, "grades"),
            Self::GradesWrite => write!(f, "grades_write"),
            Self::MailClient => write!(f, "mailclient"),
            Self::MobileNumbers => write!(f, "mobile_numbers"),
            Self::OfflineAccess => write!(f, "offline_access"),
            Self::OtherEmails => write!(f, "other_emails"),
            Self::Payments => write!(f, "payments"),
            Self::Personal => write!(f, "personal"),
            Self::Photo => write!(f, "photo"),
            Self::PlacementTests => write!(f, "placement_tests"),
            Self::SessionDebugging => write!(f, "session_debugging_perms"),
            Self::ClearanceSlips => write!(f, "slips"),
            Self::ClearanceSlipsAdmin => write!(f, "slips_admin"),
            Self::StaffPerspective => write!(f, "staff_perspective"),
            Self::StudentExams => write!(f, "student_exams"),
            Self::StudentExamsEdit => write!(f, "student_exams_write"),
            Self::Studies => write!(f, "studies"),
            Self::SurveysFilling => write!(f, "surveys_filling"),
            Self::SurveysReports => write!(f, "surveys_reports"),
            Self::ThesesProtocolsEdit => write!(f, "theses_protocols_write"),
        }
    }
}
