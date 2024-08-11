use std::str::FromStr;

/// /services/apiref/scopes
pub enum Scope {
    /// Allows access to get administration documents etc.
    AdministrativeDocuments,
    /// Provides access to user's ID cards data, such as chip uid or expiration date
    Cards,
    /// Allows you to change user preferences (via the uprefs module). You may need some other scopes in order to change or view some of the preferences. Also, the access to some important preferences may be restricted in other ways, i.e. only Administrative Consumers may be allowed to change them.
    ChangeAllPreferences,
    /// Provides access to details and results of user's course tests.
    CourseTests,
    /// Provides access to administrative housing operations on user's behalf. For more information, please visit the housing module.
    Dorms,
    /// Allows editing user's attributes (the same thet the user can edit on his USOSweb profile page).
    ChangeUserAttributes,
    /// Provides access to user's email address.
    Email,
    /// Allows access to user's preferences, push notifications, etc.
    Events,
    /// Provides access to grades information.
    Grades,
    /// Allows access to read and write exam reports.
    GradesWrite,
    /// Provides access to the mailclient module (in the name of your user). Currently only a small set of methods is available for non-administrative consumers, but this set will be growing.
    MailClient,
    /// Provides access to user's personal mobile phone number(s).
    MobileNumbers,
    /// Enables your application to perform authorized requests on behalf of the user at any time. By default, Access Tokens expire after a short time period to ensure applications only make requests on behalf of users when they are actively using the application. This scope makes Access Tokens long-lived.
    OfflineAccess,
    /// Provides access to email addresses of other users (i.e. the ones related to your user).
    OtherEmails,
    /// Allows access to your payments.
    Payments,
    /// Provides access to user's personal data, such as PESEL number, date of birth, etc.
    Personal,
    /// Provides read access to user's photo and his/her photo visibility preferences ("who can see my photo?").
    Photo,
    /// Provides access to results of user's placement tests in foreign languages.
    PlacementTests,
    /// Allows access to official permissions related to the user's session debugging rights. Allows you to get the answer to the question "Is my user permitted to debug the session of user X?". See "can_i_debug" field of the services/users/user method for more information.
    SessionDebugging, // dev only
    /// Provides access to most of the actions within the Clearance Slips module. With this scope you can view, create and edit slips, answer questions and perform any non-administrative action which the user can perform via USOSweb. You will need an additional 'slips_admin' scope if you want to manage slip templates too.
    ClearanceSlips,
    /// Provides access to template management of the "slips" module. That is, it allows you to create and edit questions, mark templates as obsolete etc.
    ClearanceSlipsAdmin,
    /// If your user is a staff member, then this scope provides access to some common student-related data usually visible only to staff members, e.g. student numbers, or broader lists of students' study programmes.
    StaffPerspective,
    /// Provides access to lists of student's exams, information on their examiners, places the exams take place etc.
    StudentExams,
    /// Allows to register and unregister the student from his exams.
    StudentExamsEdit,
    /// Provides access to lists of programmes, courses, classes and groups which the user attends (as a student).
    Studies,
    /// Allows access to surveys from students point of view.  With this scope you can fetch and fill out surveys.
    SurveysFilling,
    /// Allows access to reports on surveys that concern user as a lecturer.
    SurveysReports,
    /// Allows access to editing diploma exam protocols, e.g. signing protocols.
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
