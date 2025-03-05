use core::result;
use std::{io::ErrorKind, path::Path};

use log::{info, warn};
use native_db::{db_type::Error as NativeDbError, Builder, Database, Models};
use once_cell::sync::Lazy;
use redb::{DatabaseError, StorageError};

use crate::{models::ResoluteMod, Error, Result};

/// Wrapper for interacting with a Resolute database
#[allow(missing_debug_implementations)]
pub struct ResoluteDatabase<'a> {
	db: Database<'a>,
}

impl ResoluteDatabase<'_> {
	/// Opens a database using a provided builder.
	/// If the database doesn't already exist at the given path, it will be created.
	pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
		info!("Opening database at {}", db_path.as_ref().display());

		// Try to open an already-existing database
		let builder = Builder::new();
		let db = match builder.open(&MODELS, &db_path) {
			// If it fails because it doesn't exist, then go ahead and create one instead
			Err(
				NativeDbError::Io(err)
				| NativeDbError::RedbDatabaseError(DatabaseError::Storage(StorageError::Io(err))),
			) if err.kind() == ErrorKind::NotFound => {
				warn!("Database doesn't exist; creating");
				builder.create(&MODELS, &db_path)?
			}

			Ok(db) => db,
			Err(err) => return Err(err.into()),
		};

		// Run migrations
		#[cfg(feature = "migrate")]
		{
			info!("Running migrations on database");
			let rw = db.rw_transaction()?;
			rw.migrate::<ResoluteMod>()?;
			rw.commit()?;
		}

		info!("Database initialized");
		Ok(Self { db })
	}

	/// Retrieves all mods stored in the database
	pub fn get_mods(&self) -> Result<Vec<ResoluteMod>> {
		let read = self.db.r_transaction()?;
		let mods = read.scan().primary()?.all()?.collect::<result::Result<_, _>>()?;
		Ok(mods)
	}

	/// Retrieves all mods from the database that have an installed version
	pub fn get_installed_mods(&self) -> Result<Vec<ResoluteMod>> {
		let read = self.db.r_transaction()?;
		let mods = read
			.scan()
			.primary()?
			.all()?
			.collect::<result::Result<Vec<_>, _>>()?
			.into_iter()
			.filter(|rmod: &ResoluteMod| rmod.installed_version.is_some())
			.collect();
		Ok(mods)
	}

	/// Retrieves a single mod from the database by its ID
	pub fn get_mod(&self, id: impl AsRef<str>) -> Result<Option<ResoluteMod>> {
		let read = self.db.r_transaction()?;
		let rmod = read.get().primary(id.as_ref())?;
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
		let id = id.as_ref();
		let read = self.db.r_transaction()?;
		let rmod: ResoluteMod = read
			.get()
			.primary(id)?
			.ok_or_else(|| Error::ItemNotFound(id.to_owned()))?;

		// Remove it
		self.remove_mod(rmod)
	}
}

/// Models that a [`ResoluteDatabase`] interacts with
#[allow(clippy::absolute_paths)]
pub static MODELS: Lazy<Models> = Lazy::new(|| {
	let mut models = Models::new();

	// Current models
	models
		.define::<ResoluteMod>()
		.expect("Unable to define current ResoluteMod model");

	// Legacy models
	#[cfg(feature = "models_v1")]
	models
		.define::<crate::models::v1::ResoluteMod>()
		.expect("Unable to define v1 ResoluteMod model");

	models
});
