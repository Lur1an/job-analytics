use crate::{db::Job, JobDetails, JobPost};
use async_openai::{types::*, *};
use lazy_static::lazy_static;

pub async fn create_job(job_post: JobPost) -> Job {
    let job_details;
    if let Some(raw_data) = &job_post.raw_data {
        job_details = analyze(raw_data).await;
    } else {
        job_details = None;
    }
    Job::new(job_details, job_post)
}

const PROMPT_BASE: &str = r#"
Your task is to analyze data about job postings and reply in JSON only.
Always respond in JSON.
DO NOT make up data that is not explicitly present in the provided context.
Your JSON deserializes into the following struct:
"""
pub struct JobDetails {
    requirements: Vec<String>,
    tasks: Vec<String>,
    technologies: Vec<String>,
    benefits: Vec<String>,
    programming_languages: Vec<String>,
    salary_forecast: Option<(u32, u32)>,
    experience_level: ExperienceLevel,
    application_url: Option<String>,
}
"""
- experience_level can be one of the following values: ["Junior", "Mid", "Senior", "Lead"]
- benefits is an array of keywords, make sure to pick conventional ones 
Data:
"""
"#;

lazy_static! {
    static ref OPENAI_CLIENT: Client = Client::new();
}

async fn analyze(data: &str) -> Option<JobDetails> {
    let mut prompt = PROMPT_BASE.to_owned();
    prompt.push_str(data);
    prompt.push_str(r#""""#);
    log::debug!("Prompt: {}", prompt);
    let client = OPENAI_CLIENT.clone();
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a helpful assistant.")
                .build()
                .unwrap(),
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(prompt)
                .build()
                .unwrap(),
        ])
        .build()
        .unwrap();
    let response = client.chat().create(request).await;
    match response {
        Ok(chat_completion) => {
            log::info!("OpenAI response: {:?}", chat_completion);
            let details = &chat_completion.choices.last()?.message.content;
            let job_details = serde_json::from_str::<JobDetails>(details);
            match job_details {
                Ok(details) => Some(details),
                Err(e) => {
                    log::error!("Failed to parse response into JobDetails: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            log::error!("OpenAI error: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_gpt_jobdetails_analysys() {
        env_logger::init();
        dotenv::dotenv().ok();
        let raw_data = r#"
GehaltsspanneAngabe des Arbeitgebers 60.000 €90.000 €Alle ErgebnisseVor 4 TagenSenior Software Developer Java EE | Branchenlösungen | 60% Home-Office | bis ca.90.000€ p.a. (mwd)Vesterling AG4.5Von 395 Mitarbeitenden bewertetZur Arbeitgeber-WebsiteKölnVollzeitInternet und InformationstechnologieGehaltsspanneAngabe des Arbeitgebers 60.000 €90.000 €     Vesterling Personalberatung für Technologie:  Wir sind Pioniere im Technology Recruiting und vermitteln Informatiker und Ingenieure in Festanstellung. Wir sind für mehr als 2.500 Unternehmen tätig. Einmal bei uns bewerben, unzählige Job-Chancen erhalten.Unser Klient ist ein langjährig erfolgreich wachsendes und großes IT-Beratungsunternehmen. Sein Schwerpunkt liegt auf der Entwicklung von komplexen, individuell gestalteten Anwendungssystemen u. a. für die Branchen Finanzdienstleistung, Automotive, Logistik und Gesundheitswesen. Seinen Mitarbeitern bietet er einen finanziell attraktiven und sicheren Arbeitsplatz, klare Karriereperspektiven und sehr gute Weiterbildungs- und Entwicklungsmöglichkeiten.Technisch leitende Mitarbeit in branchenorientierten Softwareentwicklungsprojekten. Standort: Köln Vertragsart: Unbefristete Festanstellung durch unseren Klienten    Ihre Aufgaben   Als Senior Software Developer Java EE arbeiten Sie mit bei der Digitalisierung von größeren Auftraggebern und in abwechslungsreichen Projekten. Gemeinsam mit Ihren Kollegen übernehmen Sie Verantwortung für Teilsysteme und ggf. auch die technische Leitung eines Teams. Sie sind umfassend in Projekten tätig, von Analyse und Konzeption, über Programmierung und Test bis hin zur Einführung von Softwaresystemen (Client- und Server, Java, JEE, Spring). Sie sind Ansprechpartner für technische Fragen. Sie arbeiten mit bei der Erstellung von Fach- und IT-​Architekturen und beim Design der Softwaresysteme.     Ihr Profil   (Fach-) Hochschulstudium (Informatik, Wirtschaftsinformatik, BWL, Mathematik, Naturwissenschaften) oder eine vergleichbare Qualifikation Mindestens 4 Jahre Erfahrung als Software Engineer / Softwareentwickler Gute Kenntnisse in der Softwareentwicklung (Methoden, Datenbanken, Frameworks, Tools, Patterns) sowie ein gutes Verständnis von IT-​Vorgehensmodellen (V-​Modell XT, RUP, Scrum o.ä.) Gute Deutsch- und Englischkenntnisse      Machen Sie Ihren nächsten Karriereschritt und bewerben Sie sich bei uns. Ihren Wunsch nach Diskretion & DSGVO-konformem Datenschutz erfüllen wir mit äußerster Sorgfalt.        Alle neuen Jobs als Java EE per E-Mail bekommen:Suchauftrag erstellenArbeitsort51061 KölnDeutschlandArbeitgeberVesterling AG51 - 200 MitarbeitendeAlle StellenangeboteWas sagen Mitarbeitende?Gesamtbewertung4.5Basierend auf 395 BewertungenVorteile für MitarbeitendeFlexible ArbeitszeitenMit Öffis erreichbarPrivat das Internet nutzenWeiterbildungFirmen-EventsHome-Office möglichParkplatzSmartphoneGesundheits-AngeboteBarrierefreiheitFirmenwagenGewinnbeteiligungBetriebliche AltersvorsorgeBetriebsarztKantineKinderbetreuungRabatte für MitarbeitendeHunde willkommenMehr anzeigenAlle Vorteile für MitarbeitendeNeuUnternehmenskulturBasierend auf 12 BewertungenVesterling AGBranchen-DurchschnittTraditionelleKulturModerneKulturWork-Life-BalanceArbeitPrivatesUmgang miteinanderResultate erzielenZusammenarbeitenFührungRichtung vorgebenMitarbeitende beteiligenStrategische RichtungStabilität sichernVeränderungen antreibenKulturkompass: Ist das Unternehmen eher traditionell oder modern?Die Bewertung der Unternehmenskultur kommt komplett von Mitarbeitenden: Diese wählen, natürlich anonym, bis zu 40 von insgesamt 160 kulturellen Merkmalen aus, um ihre Unternehmenskultur bestmöglich zu beschreiben.12 Mitarbeitende haben abgestimmt: Sie bewerten die Unternehmenskultur bei Vesterling AG als  modern. Dies stimmt in etwa mit dem Branchen-Durchschnitt überein.Der Kulturkompass zeigt jeweils ein Gesamtergebnis sowie Details für diese Bereiche: Work-Life-Balance, Zusammenarbeit, Führung und strategische Ausrichtung.Mehr Infos direkt auf kununuDetails anzeigenFeedback Wie findest Du die Gestaltung dieser Seite?Dein Feedback hilft uns, sie Seite für Dich zu verbessern.GutGeht soNicht gutÄhnliche JobsVor 4 Tagen(Senior) Software Entwickler Java | bis 90.000 € | bis zu 60 % Home-Office möglich (mwd)KölnVesterling AG4.560.000 € – 90.000 €Vor 5 TagenSenior Softwarearchitekt Java EE / Entwickler | Inhouse / HomeOffice | bis 95.000€ (mwd)KölnVesterling AG4.570.000 € – 95.000 €Vor 18 TagenSenior Java EE Architect / Softwareentwickler (m/w/d)Bonn, München, Nürnberg, RheinbachBWI GmbH63.500 € – 75.500 €Vor 25 TagenSoftware Developer Backend (Java / .NET / Python) (m/w/d)BonnCONET4.051.000 € – 80.000 €Vor 5 TagenJava EE Entwickler | 100% Home-Office / Inhouse | Gehalt bis ca. 80.000€ p.a. (mwd)KölnVesterling AG4.560.000 € – 80.000 €Vor 10 TagenJava EE Developer/ Architekt (m/w/d)Bonn, München, Nürnberg, RheinbachBWI GmbH63.500 € – 75.500 €Vor 25 TagenSoftwareentwickler (m/w/d)KölnF mal s GmbH53.500 € – 75.500 €Vor 13 TagenFull-Stack Entwickler (m/w/d) Schwerpunkt Java / JEEKölnCMB Gastro GmbH53.500 € – 75.500 €Vor 5 TagenSoftware Developer* mit C++ und Java - Bonn - 70.000€ (*all gender)BonnNXT Hero GmbH5.053.500 € – 69.500 €
        "#;
        let job_details = analyze(raw_data).await.expect("Should have job_details");
        println!("{:?}", job_details);
    }
}
