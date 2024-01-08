use std::path::Path;

use log::info;
use native_db::{Database, DatabaseBuilder};

use crate::{mods::ResoluteMod, Error, Result};

/// Wrapper for interacting with a Resolute database
pub struct ResoluteDatabase<'a> {
	db: Database<'a>,
}

impl<'a> ResoluteDatabase<'a> {
	/// Opens a database using a provided native_db builder
	pub fn open(builder: &'a mut DatabaseBuilder, db_path: impl AsRef<Path>) -> Result<Self> {
		builder.define::<ResoluteMod>()?;
		let db = builder.create(&db_path)?;

		info!("Database initialized at {}", db_path.as_ref().display());
		Ok(Self { db })
	}

	/// Retrieves all mods stored in the database
	pub fn get_mods(&self) -> Result<Vec<ResoluteMod>> {
		let read = self.db.r_transaction()?;
		let mods = read.scan().primary()?.all().collect();
		Ok(mods)
	}

	/// Retrieves all mods from the database that have an installed version
	pub fn get_installed_mods(&self) -> Result<Vec<ResoluteMod>> {
		let read = self.db.r_transaction()?;
		let mods = read
			.scan()
			.primary()?
			.all()
			.filter(|rmod: &ResoluteMod| rmod.installed_version.is_some())
			.collect();
		Ok(mods)
	}

	/// Retrieves a single mod from the database by its ID
	pub fn get_mod(&self, id: impl AsRef<str>) -> Result<Option<ResoluteMod>> {
		let read = self.db.r_transaction()?;
		let rmod = read.get().primary(id.as_ref().to_string())?;
		Ok(rmod)
	}

	/// Stores a mod in the database (overwrites any existing entry for the same mod)
	pub fn store_mod(&self, rmod: ResoluteMod) -> Result<()> {
		let mod_name = rmod.to_string();

		let rw = self.db.rw_transaction()?;
		rw.insert(rmod)?;
		rw.commit()?;

		info!("Stored mod {} in the database", mod_name);
		Ok(())
	}

	/// Removes a mod from the database
	pub fn remove_mod(&self, rmod: ResoluteMod) -> Result<()> {
		let mod_name = rmod.to_string();

		// Remove the mod
		let rw = self.db.rw_transaction()?;
		rw.remove(rmod)?;
		rw.commit()?;

		info!("Removed mod {} from the database", mod_name);
		Ok(())
	}

	/// Removes a mod from the database by its ID
	pub fn remove_mod_by_id(&self, id: impl AsRef<str>) -> Result<()> {
		// Find the item in the database
		let id = id.as_ref().to_string();
		let read = self.db.r_transaction()?;
		let rmod: ResoluteMod = read.get().primary(id.clone())?.ok_or_else(|| Error::ItemNotFound(id))?;

		// Remove it
		self.remove_mod(rmod)
	}
}
