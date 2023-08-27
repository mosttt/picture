use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bytes::Bytes;
use ssh2::{FileStat, Session};

use crate::error::PError;
use crate::Result;

#[derive(Clone)]
pub struct SSHClient {
    sess: Session,
}

impl SSHClient {
    pub fn new(tcp: TcpStream) -> Self {
        let mut sess = Session::new().unwrap();
        sess.set_timeout(Duration::from_secs(30).as_millis() as u32);
        sess.set_tcp_stream(tcp.try_clone().unwrap());
        sess.handshake().unwrap();
        Self { sess }
    }

    pub fn auth_by_password(&self, username: &str, password: &str) -> &Self {
        self.sess.userauth_password(username, password).unwrap();
        assert!(self.sess.authenticated());
        self
    }
    pub fn exec(&self, command: &str) -> Result<String> {
        let mut channel = self.sess.channel_session()?;
        channel.exec(command)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            return Err(PError::SSH2OPError(format!("执行命令失败: {}", command)));
        }
        Ok(output)
    }
    pub fn dir_exists(&self, remote_path: impl AsRef<Path>) -> Result<bool> {
        let remote_path = remote_path.as_ref();
        // 打开一个新的SFTP会话并检查目录是否存在
        let sftp = self.sess.sftp()?;
        Ok(sftp.realpath(remote_path.as_ref()).is_ok())
    }
    pub fn file_exists(&self, remote_path: impl AsRef<Path>) -> Option<FileStat> {
        // let mut channel = self.sess.channel_session().unwrap();
        // let command = format!("test -e {}", remote_path);
        // channel.exec(&command).unwrap();
        // let mut output = Vec::new();
        // channel.read_to_end(&mut output).unwrap();
        // channel.wait_close().unwrap();
        // let exit_status = channel.exit_status().unwrap();
        // exit_status == 0
        // 开始sftp会话
        let sftp = self.sess.sftp().unwrap();

        // 检查特定文件是否存在
        match sftp.stat(remote_path.as_ref()) {
            Ok(metadata) => Some(metadata),
            Err(_) => None,
        }
    }
    pub fn read_dir(&self, path: impl AsRef<Path>) -> Result<Vec<(PathBuf, FileStat)>> {
        // 开始sftp会话
        let sftp = self.sess.sftp()?;
        // 列出特定目录中的所有文件
        let files = sftp.readdir(path.as_ref())?;
        Ok(files)
    }
    pub fn sftp(&self) -> Result<ssh2::Sftp> {
        Ok(self.sess.sftp()?)
    }

    pub fn download(&self, remote_path: impl AsRef<Path>) -> Result<Bytes> {
        let (mut remote_file, stat) = self.sess.scp_recv(remote_path.as_ref())?;
        let mut bytes = Vec::with_capacity(stat.size() as usize);
        remote_file.read_to_end(&mut bytes)?;
        Ok(Bytes::from(bytes))
    }

    pub fn upload(
        &self,
        remote_path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
        mode: i32,
    ) -> Result<()> {
        let contents = contents.as_ref();
        //let mode = 0o644;
        let mut remote_file =
            self.sess
                .scp_send(remote_path.as_ref(), mode, contents.len() as u64, None)?;
        remote_file.write_all(contents)?;
        Ok(())
    }
    // fn download(&self, remote_path: &str, local_path: &str) {
    //     let (mut remote_file, stat) = self.sess.scp_recv(Path::new(remote_path)).unwrap();
    //     let mut local_file = File::create(local_path).unwrap();
    //     let mut contents = Vec::new();
    //     remote_file.read_to_end(&mut contents).unwrap();
    //     local_file.write_all(&contents).unwrap();
    //     println!("Downloaded file with size: {}", stat.size());
    // }
    //
    // fn upload(&self, local_path: &str, remote_path: &str) {
    //         let mut local_file = File::open(local_path).unwrap();
    //         let metadata = local_file.metadata().unwrap();
    //         //let mode = metadata.permissions().mode() as i32;
    //         let mode = 0o644;
    //         let mut remote_file = self.sess.scp_send(Path::new(remote_path), mode, metadata.len(), None).unwrap();
    //         let mut contents = Vec::new();
    //         local_file.read_to_end(&mut contents).unwrap();
    //         remote_file.write_all(&contents).unwrap();
    //         println!("Uploaded file with size: {}", metadata.len());
    //
    // }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn t() {}
}
