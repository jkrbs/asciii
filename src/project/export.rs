use bill::{Bill, ItemList, Tax};
use crate::util::currency_to_string;

use crate::storage::storable::Storable;
use crate::project::Project;
use super::spec::*;
use super::computed_field::ComputedField;

pub trait ExportTarget<T> {
    fn export(&self) -> T;
}

fn opt_str(opt: Option<&str>) -> Option<String> {
    opt.map(ToOwned::to_owned)
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Client {
    title: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    full_name: Option<String>,
    address: Option<String>,
    email: Option<String>,
    addressing: Option<String>,
}

impl ExportTarget<Client> for Project {
    fn export(&self) -> Client {
        Client {
            full_name: self.client().full_name(),
            addressing: self.client().addressing(),
            email: opt_str(self.client().email().ok()),
            last_name: opt_str(self.client().last_name().ok()),
            first_name: opt_str(self.client().first_name().ok()),
            title: opt_str(self.client().title().ok()),
            address: opt_str(self.client().address().ok()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Event {
    name: Option<String>,
    date: Option<String>,
    manager: Option<String>,
}

use chrono::prelude::*;
fn dmy(date: Option<Date<Utc>>) -> Option<String> {
    date.map(|d| d.format("%d.%m.%Y").to_string())
}

impl ExportTarget<Event> for Project {
    fn export(&self) -> Event {
        Event {
            name: IsProject::name(self).ok().map(ToOwned::to_owned),
            date: dmy(self.event_date().ok()),
            manager: self.responsible().ok().map(ToOwned::to_owned),
        }
    }
}


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Service {
    time: Option<f64>,
    tax: Option<f64>,
    salary: Option<String>,
    gross_total: Option<String>,
    net_total: Option<String>,
    employees: Option<Vec<Employee>>,
}


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
/// TODO: move this type to spec
pub struct Employee {
    name: String,
    salary: String,
    time: f64,
    wage: String,
}

fn export_employee(e: &crate::project::spec::Employee) -> Employee {
    Employee {
        name: e.name.clone(),
        time: e.time,
        salary:  e.salary.postfix().to_string(),
        wage:  e.wage.postfix().to_string(),
    }
}

impl ExportTarget<Service> for Project {
    fn export(&self) -> Service {
        Service {
            time:         self.hours().total_time(),
            tax:          self.hours().tax().ok().map(|t|t.value()),
            salary:       self.hours().salary().ok()
                                      .map(|s| s.postfix().to_string()),
            gross_total:  self.hours().gross_wages()
                                      .map(|s| s.postfix().to_string()),
            net_total:    self.hours().net_wages()
                                      .map(|s| s.postfix().to_string()),
            employees:    self.hours().employees().ok()
                                      .map(|employees|
                                           employees.iter()
                                                .map(export_employee)
                                                .collect()
                                      )
        }
    }
}


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Sum {
    gross_sum: String,
    has_tax: bool,
    tax_sum: String,
    tax_value: f64,
}

use super::product::Product;
fn sums_from_bill(bill: &Bill<Product<'_>>) -> Vec<Sum> {
    bill.iter()
        .map(|(tax, list)| Sum::from_itemlist(*tax, list))
        .rev()
        .collect::<Vec<_>>()
}

impl Sum {
    pub fn from_itemlist(tax: Tax, list: &ItemList<Product<'_>>) -> Sum {
        let gross_sum = list.gross_sum();
        let tax_sum = list.tax_sum();
        Sum {
            tax_value: (tax.into_inner() * 100.0),
            gross_sum: currency_to_string(&gross_sum),
            tax_sum: currency_to_string(&tax_sum),
            has_tax: (tax.into_inner() > 0f64),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Offer {
    // appendix: Option<i64>,
    date: Option<String>,
    number: Option<String>,
    sums: Vec<Sum>,
    net_total: String,
    gross_total: String,
}


impl ExportTarget<Offer> for Project {
    fn export(&self) -> Offer {
        let (offer, _) = self.bills().unwrap();
        Offer {
            // appendix: self.offer().appendix(),
            date: dmy(self.offer().date().ok()),
            number: self.offer().number().ok(),
            sums: sums_from_bill(&offer),
            net_total: currency_to_string(&offer.net_total()),
            gross_total: currency_to_string(&offer.gross_total()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Invoice {
    date: Option<String>,
    number: Option<String>,
    number_long: Option<String>,
    official: Option<String>,
    sums: Vec<Sum>,
    net_total: String,
    gross_total: String,
}


impl ExportTarget<Invoice> for Project {
    fn export(&self) -> Invoice {
        let (_, invoice) = self.bills().unwrap();

        Invoice {
            date: dmy(self.invoice().date().ok()),
            number: self.invoice().number_str(),
            number_long: self.invoice().number_long_str(),
            official: self.invoice().official().ok(),
            sums: sums_from_bill(&invoice),
            net_total: currency_to_string(&invoice.net_total()),
            gross_total: currency_to_string(&invoice.gross_total()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct ExportProduct {
    name: String,
    price: String,
    unit: String,
    amount: f64,
    cost: String,
    tax: f64,
}

fn bill_products(bill: &Bill<Product<'_>>) -> Vec<ExportProduct> {
    bill.as_items_with_tax()
        .into_iter()
        .map(|(tax, item)| {
        ExportProduct {
            name: item.product.name.to_string(),
            price: currency_to_string(&item.product.price),
            unit: item.product.unit.unwrap_or("").to_string(),
            amount: item.amount,
            cost: currency_to_string(&item.gross()),
            tax: tax.value(),
        }
    })
        .collect()
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Bills {
    pub offer: Vec<ExportProduct>,
    pub invoice: Vec<ExportProduct>,
}


impl ExportTarget<Bills> for Project {
    fn export(&self) -> Bills {
        let (offer, invoice) = self.bills().unwrap();

        Bills {
            offer: bill_products(&offer),
            invoice: bill_products(&invoice),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Complete {
    client: Client,
    event: Event,
    service: Service,
    offer: Offer,
    invoice: Invoice,
    bills: Bills,
    checks: Checks,
    errors: Errors,
    extras: Extras,
}


impl ExportTarget<Complete> for Project {
    fn export(&self) -> Complete {
        Complete {
            client: self.export(),
            event: self.export(),
            service: self.export(),
            offer: self.export(),
            invoice: self.export(),
            bills: self.export(),
            checks: self.export(),
            errors: self.export(),
            extras: self.export(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Checks {
    missing_for_offer: bool,
    missing_for_invoice: bool,
    ready_for_archive: bool,
    payed_by_customer: bool,
    payed_employees: bool,
    canceled: bool,
}

impl ExportTarget<Checks> for Project {
    fn export(&self) -> Checks {
        Checks {
            missing_for_offer: self.is_missing_for_offer().is_empty(),
            missing_for_invoice: self.is_missing_for_invoice().is_empty(),
            ready_for_archive: self.is_ready_for_archive().is_empty(),
            payed_by_customer: self.is_payed(),
            payed_employees: self.hours().employees_payed(),
            canceled: self.canceled(),
            // errors: self.is_missing_for_offer().err().map(|list| list.errors)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Errors {
    missing_for_offer:   Vec<String>,
    missing_for_invoice: Vec<String>,
    ready_for_archive: Vec<String>,
}


impl ExportTarget<Errors> for Project {
    fn export(&self) -> Errors {
        Errors {
            missing_for_offer:   self.is_missing_for_offer(),
            missing_for_invoice: self.is_missing_for_invoice(),
            ready_for_archive: self.is_ready_for_archive(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Extras {
    dir: Option<String>,
    age: Option<i64>,
    our_bad: Option<i64>,
    their_bad: Option<i64>,
    sort_index: Option<String>,
}

impl ExportTarget<Extras> for Project {
    fn export(&self) -> Extras {
        Extras {
            dir: ComputedField::Dir.get(self),
            age: self.age(),
            our_bad: self.our_bad().map(|d| d.num_days()),
            their_bad: self.their_bad().map(|d| d.num_days()),
            sort_index: self.index(),
        }
    }
}
