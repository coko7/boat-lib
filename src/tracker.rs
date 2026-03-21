use anyhow::{Context, Result, bail, ensure};
use chrono::Local;

use crate::models::{Activity, TrackingEntry};
use crate::repository::{Id, Repository, RepositoryItem};

#[derive(Debug)]
pub struct SimpleTracker {
    activity_repo: Box<dyn Repository<Activity>>,
    tracking_repo: Box<dyn Repository<TrackingEntry>>,
}

trait ActivityTracker {
    fn create(&mut self, new: Activity) -> Result<Id>;
    fn update(&mut self, updated: Activity) -> Result<()>;
    fn remove(&mut self, activity: Id) -> Result<()>;
    fn get_current(&self) -> Result<Option<Activity>>;
    fn start(&mut self, activity: Id) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

impl ActivityTracker for SimpleTracker {
    fn create(&mut self, item: Activity) -> Result<Id> {
        ensure!(
            item.id().is_empty(),
            "the ID of a new activity should not be set!"
        );
        ensure!(
            !item.name().is_empty(),
            "activity must have a non-empty name"
        );

        let id = self.activity_repo.create(item)?;
        Ok(id)
    }

    fn update(&mut self, updated: Activity) -> Result<()> {
        ensure!(!updated.id().is_empty(), "updated activity must have an ID");
        let id = updated.id();
        let _ = self.get_activity_by_id(id)?;
        self.activity_repo.update(updated)
    }

    fn remove(&mut self, activity: Id) -> Result<()> {
        let _ = self.get_activity_by_id(activity.clone())?;
        self.activity_repo.delete(activity)
    }

    fn get_current(&self) -> Result<Option<Activity>> {
        let ongoing_tracking = match self.get_ongoing_tracking()? {
            Some(val) => val,
            None => return Ok(None),
        };

        let activity = ongoing_tracking.activity_id;
        let activity = self.get_activity_by_id(activity)?;

        Ok(Some(activity))
    }

    fn start(&mut self, activity: Id) -> Result<()> {
        let _ = self.get_activity_by_id(activity.clone())?;

        let now = Local::now();
        if let Some(mut ongoing) = self.get_ongoing_tracking()? {
            ongoing.end = Some(now);
            let _ = self.tracking_repo.update(ongoing);
        }

        let niu_tracking = TrackingEntry::new(activity, now);
        self.tracking_repo.create(niu_tracking)?;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let now = Local::now();
        if let Some(mut ongoing) = self.get_ongoing_tracking()? {
            ongoing.end = Some(now);
            let _ = self.tracking_repo.update(ongoing);
        }
        Ok(())
    }
}

impl SimpleTracker {
    pub fn new(
        activity_repo: Box<dyn Repository<Activity>>,
        tracking_repo: Box<dyn Repository<TrackingEntry>>,
    ) -> Self {
        Self {
            activity_repo,
            tracking_repo,
        }
    }

    fn get_activity_by_id(&self, id: Id) -> Result<Activity> {
        self.activity_repo
            .get(id.clone())?
            .with_context(|| format!("activity not found: {id}"))
    }

    fn get_ongoing_tracking(&self) -> Result<Option<TrackingEntry>> {
        let all = self.tracking_repo.list()?;
        let mut ongoing_iter = all.into_iter().filter(|t| t.end.is_none());

        let first = ongoing_iter.next();
        let second = ongoing_iter.next();
        if second.is_some() {
            bail!("there should be only one ongoing activity!");
        }

        Ok(first)
    }
}
