/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use std_prelude::*;
use walkdir;
use path_abs::*;

pub trait PathDirWalk {
    fn walk(&self) -> WalkBuild;
}

impl PathDirWalk for PathDir {
    /// Similar to [`list`] except uses walks the directory recursively and with extreme
    /// performance (using `walkdir` under the hood).
    ///
    /// [`list`]: TODO
    fn walk(&self) -> WalkBuild {
        WalkBuild::new(self.clone(), walkdir::WalkDir::new(&self))
    }
}

/// A builder to create an iterator for recursively walking a directory.
///
/// Results are returned in depth first fashion, with directories yielded
/// before their contents. If [`contents_first`] is true, contents are yielded
/// before their directories. The order is unspecified but if [`sort_by`] is
/// given, directory entries are sorted according to this function. Directory
/// entries `.` and `..` are always omitted.
///
/// If an error occurs at any point during iteration, then it is returned in
/// place of its corresponding directory entry and iteration continues as
/// normal. If an error occurs while opening a directory for reading, then it
/// is not descended into (but the error is still yielded by the iterator).
/// Iteration may be stopped at any time. When the iterator is destroyed, all
/// resources associated with it are freed.
///
/// [`contents_first`]: struct.WalkBuild.html#method.contents_first
/// [`sort_by`]: struct.WalkBuild.html#method.sort_by
///
/// TODO: copy/paste additional docs
pub struct WalkBuild {
    path: PathDir,
    walk: walkdir::WalkDir
}

impl WalkBuild {
    fn new(path: PathDir, walk: walkdir::WalkDir) -> WalkBuild {
        WalkBuild {
            path: path,
            walk: walk,
        }
    }

    /// Set the minimum depth of entries yielded by the iterator.
    ///
    /// The smallest depth is `0` and always corresponds to the path given
    /// to the `new` function on this type. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    pub fn min_depth(self, depth: usize) -> Self {
        WalkBuild::new(self.path, self.walk.min_depth(depth))
    }

    /// Set the maximum depth of entries yield by the iterator.
    ///
    /// The smallest depth is `0` and always corresponds to the path given
    /// to the `new` function on this type. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    ///
    /// Note that this will not simply filter the entries of the iterator, but
    /// it will actually avoid descending into directories when the depth is
    /// exceeded.
    pub fn max_depth(self, depth: usize) -> Self {
        WalkBuild::new(self.path, self.walk.max_depth(depth))
    }

    /// Follow symbolic links. By default, this is disabled.
    ///
    /// When `yes` is `true`, symbolic links are followed as if they were
    /// normal directories and files. If a symbolic link is broken or is
    /// involved in a loop, an error is yielded.
    ///
    /// When enabled, the yielded [`DirEntry`] values represent the target of
    /// the link while the path corresponds to the link. See the [`DirEntry`]
    /// type for more details.
    ///
    /// [`DirEntry`]: struct.DirEntry.html
    pub fn follow_links(self, yes: bool) -> Self {
        WalkBuild::new(self.path, self.walk.follow_links(yes))
    }

    /// Set the maximum number of simultaneously open file descriptors used
    /// by the iterator.
    ///
    /// `n` must be greater than or equal to `1`. If `n` is `0`, then it is set
    /// to `1` automatically. If this is not set, then it defaults to some
    /// reasonably low number.
    ///
    /// This setting has no impact on the results yielded by the iterator
    /// (even when `n` is `1`). Instead, this setting represents a trade off
    /// between scarce resources (file descriptors) and memory. Namely, when
    /// the maximum number of file descriptors is reached and a new directory
    /// needs to be opened to continue iteration, then a previous directory
    /// handle is closed and has its unyielded entries stored in memory. In
    /// practice, this is a satisfying trade off because it scales with respect
    /// to the *depth* of your file tree. Therefore, low values (even `1`) are
    /// acceptable.
    ///
    /// Note that this value does not impact the number of system calls made by
    /// an exhausted iterator.
    ///
    /// # Platform behavior
    ///
    /// On Windows, if `follow_links` is enabled, then this limit is not
    /// respected. In particular, the maximum number of file descriptors opened
    /// is proportional to the depth of the directory tree traversed.
    pub fn max_open(self, n: usize) -> Self {
        WalkBuild::new(self.path, self.walk.max_open(n))
    }

    /// Set a function for sorting directory entries.
    ///
    /// If a compare function is set, the resulting iterator will return all
    /// paths in sorted order. The compare function will be called to compare
    /// entries from the same directory.
    ///
    /// ```rust,no-run
    /// use std::cmp;
    /// use std::ffi::OsString;
    /// use walkdir::WalkDir;
    ///
    /// WalkDir::new("foo").sort_by(|a,b| a.file_name().cmp(b.file_name()));
    /// ```
    pub fn sort_by<F>(self, cmp: F) -> Self
    where F: FnMut(&walkdir::DirEntry, &walkdir::DirEntry) -> Ordering + Send + Sync + 'static
    {
        WalkBuild::new(self.path, self.walk.sort_by(cmp))
    }

    /// Yield a directory's contents before the directory itself. By default,
    /// this is disabled.
    ///
    /// When `yes` is `false` (as is the default), the directory is yielded
    /// before its contents are read. This is useful when, e.g. you want to
    /// skip processing of some directories.
    ///
    /// When `yes` is `true`, the iterator yields the contents of a directory
    /// before yielding the directory itself. This is useful when, e.g. you
    /// want to recursively delete a directory.
    ///
    /// # Example
    ///
    /// Assume the following directory tree:
    ///
    /// ```text
    /// foo/
    ///   abc/
    ///     qrs
    ///     tuv
    ///   def/
    /// ```
    ///
    /// With contents_first disabled (the default), the following code visits
    /// the directory tree in depth-first order:
    ///
    /// ```no_run
    /// use walkdir::WalkDir;
    ///
    /// for entry in WalkDir::new("foo") {
    ///     let entry = entry.unwrap();
    ///     println!("{}", entry.path().display());
    /// }
    ///
    /// // foo
    /// // abc
    /// // qrs
    /// // tuv
    /// // def
    /// ```
    ///
    /// With contents_first enabled:
    ///
    /// ```no_run
    /// use walkdir::WalkDir;
    ///
    /// for entry in WalkDir::new("foo").contents_first(true) {
    ///     let entry = entry.unwrap();
    ///     println!("{}", entry.path().display());
    /// }
    ///
    /// // qrs
    /// // tuv
    /// // abc
    /// // def
    /// // foo
    /// ```
    pub fn contents_first(mut self, yes: bool) -> Self {
        WalkBuild::new(self.path, self.walk.contents_first(yes))
    }
}

/// A path into an entry while calling `PathDir::walk()`.
pub struct PathDirEntry {
    ty: PathType,
    entry: walkdir::DirEntry,
}

impl PathDirEntry {
    fn new(entry: walkdir::DirEntry) -> Result<PathDirEntry> {
        // TODO: the file_type is already gotten, need an "unsafe" method
        // to force-construct types
        // let abs = PathAbs::new(entry.path())?;
        // let ty = entry.file_type();
        // let ty = if ty.is_file() {
        //     PathType::File(PathFile::from_abs_unchecked(abs))
        // } else if ty.is_dir() {
        //     PathType::Dir(PathDir::from_abs_unchecked(abs)?)
        // } else {
        //     PathDir::from_abs(abs)?;
        // };
        Ok(PathDirEntry {
            ty: PathType::new(entry.path())?,
            entry: entry,
        })
    }

    /// Convert this entry into its `PathType`
    pub fn to_type(&self) -> PathType {
        self.ty.clone()
    }

    /// Returns `true` if and only if this entry was created from a symbolic
    /// link. This is unaffected by the [`follow_links`] setting.
    ///
    /// [`follow_links`]: struct.PathDirWalk.html#method.follow_links
    pub fn was_symlink(&self) -> bool {
        self.entry.path_is_symlink()
    }

    /// Returns the depth at which this entry was created relative to the root.
    ///
    /// The smallest depth is `0` and always corresponds to the path given
    /// to the `new` function on `WalkDir`. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    pub fn depth(&self) -> usize {
        self.entry.depth()
    }
}

impl AsRef<PathType> for PathDirEntry {
    fn as_ref(&self) -> &PathType {
        &self.ty
    }
}

impl Deref for PathDirEntry {
    type Target = PathType;

    fn deref(&self) -> &PathType {
        &self.ty
    }
}

impl Into<PathType> for PathDirEntry {
    /// Consume the entry, converting into the `PathType`
    ///
    /// Alternatively use [`to_type()`] if you want to preserve the entry.
    ///
    /// [`to_type()`]: struct.PathDirEntry.html#method.to_type
    fn into(self) -> PathType {
        self.ty
    }
}
