use ffi_toolkit::{
    c_str_to_pbuf, catch_panic_response, raw_ptr, rust_str_to_c_str,
};
use filecoin_proofs_api_v2::seal::SealPreCommitPhase2Output;
use filecoin_proofs_api_v2::{
    PieceInfo, RegisteredPoStProof, RegisteredSealProof, SectorId, UnpaddedByteIndex,
    UnpaddedBytesAmount,
};
use log::info;
use std::mem;
use std::path::PathBuf;
use std::slice::from_raw_parts;

use super::helpers::{c_to_rust_post_proofs_v2, to_private_replica_info_map_v2};
use super::types::*;
use crate::util::api::init_log_v2;

/// TODO: document
///
#[no_mangle]
#[cfg(not(target_os = "windows"))]
pub unsafe extern "C" fn fil_write_with_alignment_v2(
    registered_proof: fil_RegisteredSealProofV2,
    src_fd: libc::c_int,
    src_size: u64,
    dst_fd: libc::c_int,
    existing_piece_sizes_ptr: *const u64,
    existing_piece_sizes_len: libc::size_t,
) -> *mut fil_WriteWithAlignmentResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("write_with_alignment_v2: start");

        let mut response = fil_WriteWithAlignmentResponseV2::default();

        let piece_sizes: Vec<UnpaddedBytesAmount> =
            from_raw_parts(existing_piece_sizes_ptr, existing_piece_sizes_len)
                .iter()
                .map(|n| UnpaddedBytesAmount(*n))
                .collect();

        let n = UnpaddedBytesAmount(src_size);

        match filecoin_proofs_api_v2::seal::add_piece(
            registered_proof.into(),
            FileDescriptorRefV2::new(src_fd),
            FileDescriptorRefV2::new(dst_fd),
            n,
            &piece_sizes,
        ) {
            Ok((info, written)) => {
                response.comm_p = info.commitment;
                response.left_alignment_unpadded = (written - n).into();
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.total_write_unpadded = written.into();
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("write_with_alignment_v2: finish");

        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
#[cfg(not(target_os = "windows"))]
pub unsafe extern "C" fn fil_write_without_alignment_v2(
    registered_proof: fil_RegisteredSealProofV2,
    src_fd: libc::c_int,
    src_size: u64,
    dst_fd: libc::c_int,
) -> *mut fil_WriteWithoutAlignmentResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("write_without_alignment_v2: start");

        let mut response = fil_WriteWithoutAlignmentResponseV2::default();

        match filecoin_proofs_api_v2::seal::write_and_preprocess(
            registered_proof.into(),
            FileDescriptorRefV2::new(src_fd),
            FileDescriptorRefV2::new(dst_fd),
            UnpaddedBytesAmount(src_size),
        ) {
            Ok((info, written)) => {
                response.comm_p = info.commitment;
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.total_write_unpadded = written.into();
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("write_without_alignment_v2: finish");

        raw_ptr(response)
    })
}

#[no_mangle]
pub unsafe extern "C" fn fil_fauxrep_v2(
    registered_proof: fil_RegisteredSealProofV2,
    cache_dir_path: *const libc::c_char,
    sealed_sector_path: *const libc::c_char,
) -> *mut fil_FauxRepResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("fauxrep_v2: start");

        let mut response: fil_FauxRepResponseV2 = Default::default();

        let result = filecoin_proofs_api_v2::seal::fauxrep(
            registered_proof.into(),
            c_str_to_pbuf(cache_dir_path),
            c_str_to_pbuf(sealed_sector_path),
        );

        match result {
            Ok(output) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.commitment = output;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("fauxrep_v2: finish");

        raw_ptr(response)
    })
}

#[no_mangle]
pub unsafe extern "C" fn fil_fauxrep2_v2(
    registered_proof: fil_RegisteredSealProofV2,
    cache_dir_path: *const libc::c_char,
    existing_p_aux_path: *const libc::c_char,
) -> *mut fil_FauxRepResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("fauxrep2_v2: start");

        let mut response: fil_FauxRepResponseV2 = Default::default();

        let result = filecoin_proofs_api_v2::seal::fauxrep2(
            registered_proof.into(),
            c_str_to_pbuf(cache_dir_path),
            c_str_to_pbuf(existing_p_aux_path),
        );

        match result {
            Ok(output) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.commitment = output;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("fauxrep2_v2: finish");

        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
pub unsafe extern "C" fn fil_seal_pre_commit_phase1_v2(
    registered_proof: fil_RegisteredSealProofV2,
    cache_dir_path: *const libc::c_char,
    staged_sector_path: *const libc::c_char,
    sealed_sector_path: *const libc::c_char,
    sector_id: u64,
    prover_id: fil_32ByteArrayV2,
    ticket: fil_32ByteArrayV2,
    pieces_ptr: *const fil_PublicPieceInfoV2,
    pieces_len: libc::size_t,
) -> *mut fil_SealPreCommitPhase1ResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("seal_pre_commit_phase1_v2: start");

        let public_pieces: Vec<PieceInfo> = from_raw_parts(pieces_ptr, pieces_len)
            .iter()
            .cloned()
            .map(Into::into)
            .collect();

        let mut response: fil_SealPreCommitPhase1ResponseV2 = Default::default();

        let result = filecoin_proofs_api_v2::seal::seal_pre_commit_phase1(
            registered_proof.into(),
            c_str_to_pbuf(cache_dir_path),
            c_str_to_pbuf(staged_sector_path),
            c_str_to_pbuf(sealed_sector_path),
            prover_id.inner,
            SectorId::from(sector_id),
            ticket.inner,
            &public_pieces,
        )
        .and_then(|output| serde_json::to_vec(&output).map_err(Into::into));

        match result {
            Ok(output) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.seal_pre_commit_phase1_output_ptr = output.as_ptr();
                response.seal_pre_commit_phase1_output_len = output.len();
                mem::forget(output);
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("seal_pre_commit_phase1_v2: finish");

        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
pub unsafe extern "C" fn fil_seal_pre_commit_phase2_v2(
    seal_pre_commit_phase1_output_ptr: *const u8,
    seal_pre_commit_phase1_output_len: libc::size_t,
    cache_dir_path: *const libc::c_char,
    sealed_sector_path: *const libc::c_char,
) -> *mut fil_SealPreCommitPhase2ResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("seal_pre_commit_phase2_v2: start");

        let mut response: fil_SealPreCommitPhase2ResponseV2 = Default::default();

        let phase_1_output = serde_json::from_slice(from_raw_parts(
            seal_pre_commit_phase1_output_ptr,
            seal_pre_commit_phase1_output_len,
        ))
        .map_err(Into::into);

        let result = phase_1_output.and_then(|o| {
            filecoin_proofs_api_v2::seal::seal_pre_commit_phase2::<PathBuf, PathBuf>(
                o,
                c_str_to_pbuf(cache_dir_path),
                c_str_to_pbuf(sealed_sector_path),
            )
        });

        match result {
            Ok(output) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.comm_r = output.comm_r;
                response.comm_d = output.comm_d;
                response.registered_proof = output.registered_proof.into();
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("seal_pre_commit_phase2_v2: finish");

        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
pub unsafe extern "C" fn fil_seal_commit_phase1_v2(
    registered_proof: fil_RegisteredSealProofV2,
    comm_r: fil_32ByteArrayV2,
    comm_d: fil_32ByteArrayV2,
    cache_dir_path: *const libc::c_char,
    replica_path: *const libc::c_char,
    sector_id: u64,
    prover_id: fil_32ByteArrayV2,
    ticket: fil_32ByteArrayV2,
    seed: fil_32ByteArrayV2,
    pieces_ptr: *const fil_PublicPieceInfoV2,
    pieces_len: libc::size_t,
) -> *mut fil_SealCommitPhase1ResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("seal_commit_phase1_v2: start");

        let mut response = fil_SealCommitPhase1ResponseV2::default();

        let spcp2o = SealPreCommitPhase2Output {
            registered_proof: registered_proof.into(),
            comm_r: comm_r.inner,
            comm_d: comm_d.inner,
        };

        let public_pieces: Vec<PieceInfo> = from_raw_parts(pieces_ptr, pieces_len)
            .iter()
            .cloned()
            .map(Into::into)
            .collect();

        let result = filecoin_proofs_api_v2::seal::seal_commit_phase1(
            c_str_to_pbuf(cache_dir_path),
            c_str_to_pbuf(replica_path),
            prover_id.inner,
            SectorId::from(sector_id),
            ticket.inner,
            seed.inner,
            spcp2o,
            &public_pieces,
        );

        match result.and_then(|output| serde_json::to_vec(&output).map_err(Into::into)) {
            Ok(output) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.seal_commit_phase1_output_ptr = output.as_ptr();
                response.seal_commit_phase1_output_len = output.len();
                mem::forget(output);
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("seal_commit_phase1_v2: finish");

        raw_ptr(response)
    })
}

#[no_mangle]
pub unsafe extern "C" fn fil_seal_commit_phase2_v2(
    seal_commit_phase1_output_ptr: *const u8,
    seal_commit_phase1_output_len: libc::size_t,
    sector_id: u64,
    prover_id: fil_32ByteArrayV2,
) -> *mut fil_SealCommitPhase2ResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("seal_commit_phase2_v2: start");

        let mut response = fil_SealCommitPhase2ResponseV2::default();

        let scp1o = serde_json::from_slice(from_raw_parts(
            seal_commit_phase1_output_ptr,
            seal_commit_phase1_output_len,
        ))
        .map_err(Into::into);

        let result = scp1o.and_then(|o| {
            filecoin_proofs_api_v2::seal::seal_commit_phase2(
                o,
                prover_id.inner,
                SectorId::from(sector_id),
            )
        });

        match result {
            Ok(output) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.proof_ptr = output.proof.as_ptr();
                response.proof_len = output.proof.len();
                mem::forget(output.proof);
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("seal_commit_phase2_v2: finish");

        raw_ptr(response)
    })
}

/// TODO: document
#[no_mangle]
pub unsafe extern "C" fn fil_unseal_range_v2(
    registered_proof: fil_RegisteredSealProofV2,
    cache_dir_path: *const libc::c_char,
    sealed_sector_fd_raw: libc::c_int,
    unseal_output_fd_raw: libc::c_int,
    sector_id: u64,
    prover_id: fil_32ByteArrayV2,
    ticket: fil_32ByteArrayV2,
    comm_d: fil_32ByteArrayV2,
    unpadded_byte_index: u64,
    unpadded_bytes_amount: u64,
) -> *mut fil_UnsealRangeResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("unseal_range_v2: start");

        use std::os::unix::io::{FromRawFd, IntoRawFd};

        let mut sealed_sector = std::fs::File::from_raw_fd(sealed_sector_fd_raw);
        let mut unseal_output = std::fs::File::from_raw_fd(unseal_output_fd_raw);

        let result = filecoin_proofs_api_v2::seal::unseal_range(
            registered_proof.into(),
            c_str_to_pbuf(cache_dir_path),
            &mut sealed_sector,
            &mut unseal_output,
            prover_id.inner,
            SectorId::from(sector_id),
            comm_d.inner,
            ticket.inner,
            UnpaddedByteIndex(unpadded_byte_index),
            UnpaddedBytesAmount(unpadded_bytes_amount),
        );

        // keep all file descriptors alive until unseal_range returns
        let _ = sealed_sector.into_raw_fd();
        let _ = unseal_output.into_raw_fd();

        let mut response = fil_UnsealRangeResponseV2::default();

        match result {
            Ok(_) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        };

        info!("unseal_range_v2: finish");

        raw_ptr(response)
    })
}

/// Verifies the output of seal.
///
#[no_mangle]
pub unsafe extern "C" fn fil_verify_seal_v2(
    registered_proof: fil_RegisteredSealProofV2,
    comm_r: fil_32ByteArrayV2,
    comm_d: fil_32ByteArrayV2,
    prover_id: fil_32ByteArrayV2,
    ticket: fil_32ByteArrayV2,
    seed: fil_32ByteArrayV2,
    sector_id: u64,
    proof_ptr: *const u8,
    proof_len: libc::size_t,
) -> *mut super::types::fil_VerifySealResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("verify_seal_v2: start");

        let mut proof_bytes: Vec<u8> = vec![0; proof_len];
        proof_bytes.clone_from_slice(from_raw_parts(proof_ptr, proof_len));

        let result = filecoin_proofs_api_v2::seal::verify_seal(
            registered_proof.into(),
            comm_r.inner,
            comm_d.inner,
            prover_id.inner,
            SectorId::from(sector_id),
            ticket.inner,
            seed.inner,
            &proof_bytes,
        );

        let mut response = fil_VerifySealResponseV2::default();

        match result {
            Ok(true) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.is_valid = true;
            }
            Ok(false) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.is_valid = false;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        };

        info!("verify_seal_v2: finish");

        raw_ptr(response)
    })
}

/// Verifies that a proof-of-spacetime is valid.
#[no_mangle]
pub unsafe extern "C" fn fil_verify_winning_post_v2(
    randomness: fil_32ByteArrayV2,
    replicas_ptr: *const fil_PublicReplicaInfoV2,
    replicas_len: libc::size_t,
    proofs_ptr: *const fil_PoStProofV2,
    proofs_len: libc::size_t,
    prover_id: fil_32ByteArrayV2,
) -> *mut fil_VerifyWinningPoStResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("verify_winning_post_v2: start");

        let mut response = fil_VerifyWinningPoStResponseV2::default();

        let convert = super::helpers::to_public_replica_info_map_v2(replicas_ptr, replicas_len);

        let result = convert.and_then(|replicas| {
            let post_proofs = c_to_rust_post_proofs_v2(proofs_ptr, proofs_len)?;
            let proofs: Vec<u8> = post_proofs.iter().flat_map(|pp| pp.clone().proof).collect();

            filecoin_proofs_api_v2::post::verify_winning_post(
                &randomness.inner,
                &proofs,
                &replicas,
                prover_id.inner,
            )
        });

        match result {
            Ok(is_valid) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.is_valid = is_valid;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        };

        info!("verify_winning_post_v2: finish");
        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
pub unsafe extern "C" fn fil_generate_window_post_v2(
    randomness: fil_32ByteArrayV2,
    replicas_ptr: *const fil_PrivateReplicaInfoV2,
    replicas_len: libc::size_t,
    prover_id: fil_32ByteArrayV2,
) -> *mut fil_GenerateWindowPoStResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("generate_window_post_v2: start");

        let mut response = fil_GenerateWindowPoStResponseV2::default();

        let result = to_private_replica_info_map_v2(replicas_ptr, replicas_len).and_then(|rs| {
            filecoin_proofs_api_v2::post::generate_window_post(&randomness.inner, &rs, prover_id.inner)
        });

        match result {
            Ok(output) => {
                let mapped: Vec<fil_PoStProofV2> = output
                    .iter()
                    .cloned()
                    .map(|(t, proof)| {
                        let out = fil_PoStProofV2 {
                            registered_proof: (t).into(),
                            proof_len: proof.len(),
                            proof_ptr: proof.as_ptr(),
                        };

                        mem::forget(proof);

                        out
                    })
                    .collect();

                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.proofs_ptr = mapped.as_ptr();
                response.proofs_len = mapped.len();
                mem::forget(mapped);
            }
            Err(err) => {
                // If there were faulty sectors, add them to the response
                if let Some(filecoin_proofs_api_v2::StorageProofsError::FaultySectors(sectors)) =
                    err.downcast_ref::<filecoin_proofs_api_v2::StorageProofsError>()
                {
                    let sectors_u64 = sectors
                        .iter()
                        .map(|sector| u64::from(*sector))
                        .collect::<Vec<u64>>();
                    response.faulty_sectors_len = sectors_u64.len();
                    response.faulty_sectors_ptr = sectors_u64.as_ptr();
                    mem::forget(sectors_u64)
                }

                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("generate_window_post_v2: finish");

        raw_ptr(response)
    })
}

/// Verifies that a proof-of-spacetime is valid.
#[no_mangle]
pub unsafe extern "C" fn fil_verify_window_post_v2(
    randomness: fil_32ByteArrayV2,
    replicas_ptr: *const fil_PublicReplicaInfoV2,
    replicas_len: libc::size_t,
    proofs_ptr: *const fil_PoStProofV2,
    proofs_len: libc::size_t,
    prover_id: fil_32ByteArrayV2,
) -> *mut fil_VerifyWindowPoStResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("verify_window_post_v2: start");

        let mut response = fil_VerifyWindowPoStResponseV2::default();

        let convert = super::helpers::to_public_replica_info_map_v2(replicas_ptr, replicas_len);

        let result = convert.and_then(|replicas| {
            let post_proofs = c_to_rust_post_proofs_v2(proofs_ptr, proofs_len)?;

            let proofs: Vec<(RegisteredPoStProof, &[u8])> = post_proofs
                .iter()
                .map(|x| (x.registered_proof, x.proof.as_ref()))
                .collect();

            filecoin_proofs_api_v2::post::verify_window_post(
                &randomness.inner,
                &proofs,
                &replicas,
                prover_id.inner,
            )
        });

        match result {
            Ok(is_valid) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.is_valid = is_valid;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        };

        info!("verify_window_post_v2: finish");

        raw_ptr(response)
    })
}

/// Returns the merkle root for a piece after piece padding and alignment.
/// The caller is responsible for closing the passed in file descriptor.
#[no_mangle]
#[cfg(not(target_os = "windows"))]
pub unsafe extern "C" fn fil_generate_piece_commitment_v2(
    registered_proof: fil_RegisteredSealProofV2,
    piece_fd_raw: libc::c_int,
    unpadded_piece_size: u64,
) -> *mut fil_GeneratePieceCommitmentResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        use std::os::unix::io::{FromRawFd, IntoRawFd};

        let mut piece_file = std::fs::File::from_raw_fd(piece_fd_raw);

        let unpadded_piece_size = UnpaddedBytesAmount(unpadded_piece_size);
        let result = filecoin_proofs_api_v2::seal::generate_piece_commitment(
            registered_proof.into(),
            &mut piece_file,
            unpadded_piece_size,
        );

        // avoid dropping the File which closes it
        let _ = piece_file.into_raw_fd();

        let mut response = fil_GeneratePieceCommitmentResponseV2::default();

        match result {
            Ok(meta) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.comm_p = meta.commitment;
                response.num_bytes_aligned = meta.size.into();
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        raw_ptr(response)
    })
}

/// Returns the merkle root for a sector containing the provided pieces.
#[no_mangle]
pub unsafe extern "C" fn fil_generate_data_commitment_v2(
    registered_proof: fil_RegisteredSealProofV2,
    pieces_ptr: *const fil_PublicPieceInfoV2,
    pieces_len: libc::size_t,
) -> *mut fil_GenerateDataCommitmentResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("generate_data_commitment_v2: start");

        let public_pieces: Vec<PieceInfo> = from_raw_parts(pieces_ptr, pieces_len)
            .iter()
            .cloned()
            .map(Into::into)
            .collect();

        let result =
            filecoin_proofs_api_v2::seal::compute_comm_d(registered_proof.into(), &public_pieces);

        let mut response = fil_GenerateDataCommitmentResponseV2::default();

        match result {
            Ok(commitment) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.comm_d = commitment;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("generate_data_commitment_v2: finish");

        raw_ptr(response)
    })
}

#[no_mangle]
pub unsafe extern "C" fn fil_clear_cache_v2(
    sector_size: u64,
    cache_dir_path: *const libc::c_char,
) -> *mut fil_ClearCacheResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        let result =
            filecoin_proofs_api_v2::seal::clear_cache(sector_size, &c_str_to_pbuf(cache_dir_path));

        let mut response = fil_ClearCacheResponseV2::default();

        match result {
            Ok(_) => {
                response.status_code = FCPResponseStatusV2::FCPNoError;
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        };

        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
pub unsafe extern "C" fn fil_generate_winning_post_sector_challenge_v2(
    registered_proof: fil_RegisteredPoStProofV2,
    randomness: fil_32ByteArrayV2,
    sector_set_len: u64,
    prover_id: fil_32ByteArrayV2,
) -> *mut fil_GenerateWinningPoStSectorChallengeV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("generate_winning_post_sector_challenge_v2: start");

        let mut response = fil_GenerateWinningPoStSectorChallengeV2::default();

        let result = filecoin_proofs_api_v2::post::generate_winning_post_sector_challenge(
            registered_proof.into(),
            &randomness.inner,
            sector_set_len,
            prover_id.inner,
        );

        match result {
            Ok(output) => {
                let mapped: Vec<u64> = output.into_iter().map(u64::from).collect();

                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.ids_ptr = mapped.as_ptr();
                response.ids_len = mapped.len();
                mem::forget(mapped);
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("generate_winning_post_sector_challenge_v2: finish");

        raw_ptr(response)
    })
}

/// TODO: document
///
#[no_mangle]
pub unsafe extern "C" fn fil_generate_winning_post_v2(
    randomness: fil_32ByteArrayV2,
    replicas_ptr: *const fil_PrivateReplicaInfoV2,
    replicas_len: libc::size_t,
    prover_id: fil_32ByteArrayV2,
) -> *mut fil_GenerateWinningPoStResponseV2 {
    catch_panic_response(|| {
        init_log_v2();

        info!("generate_winning_post_v2: start");

        let mut response = fil_GenerateWinningPoStResponseV2::default();

        let result = to_private_replica_info_map_v2(replicas_ptr, replicas_len).and_then(|rs| {
            filecoin_proofs_api_v2::post::generate_winning_post(
                &randomness.inner,
                &rs,
                prover_id.inner,
            )
        });

        match result {
            Ok(output) => {
                let mapped: Vec<fil_PoStProofV2> = output
                    .iter()
                    .cloned()
                    .map(|(t, proof)| {
                        let out = fil_PoStProofV2 {
                            registered_proof: (t).into(),
                            proof_len: proof.len(),
                            proof_ptr: proof.as_ptr(),
                        };

                        println!("[V2] GENERATED PROOF {:?}", proof);
                        mem::forget(proof);

                        out
                    })
                    .collect();

                response.status_code = FCPResponseStatusV2::FCPNoError;
                response.proofs_ptr = mapped.as_ptr();
                response.proofs_len = mapped.len();
                mem::forget(mapped);
            }
            Err(err) => {
                response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
                response.error_msg = rust_str_to_c_str(format!("{:?}", err));
            }
        }

        info!("generate_winning_post_v2: finish");

        raw_ptr(response)
    })
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_write_with_alignment_response_v2(
    ptr: *mut fil_WriteWithAlignmentResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_write_without_alignment_response_v2(
    ptr: *mut fil_WriteWithoutAlignmentResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_fauxrep_response_v2(ptr: *mut fil_FauxRepResponseV2) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_seal_pre_commit_phase1_response_v2(
    ptr: *mut fil_SealPreCommitPhase1ResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_seal_pre_commit_phase2_response_v2(
    ptr: *mut fil_SealPreCommitPhase2ResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_seal_commit_phase1_response_v2(
    ptr: *mut fil_SealCommitPhase1ResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_seal_commit_phase2_response_v2(
    ptr: *mut fil_SealCommitPhase2ResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_unseal_range_response_v2(ptr: *mut fil_UnsealRangeResponseV2) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_generate_piece_commitment_response_v2(
    ptr: *mut fil_GeneratePieceCommitmentResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_generate_data_commitment_response_v2(
    ptr: *mut fil_GenerateDataCommitmentResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_string_response_v2(ptr: *mut fil_StringResponseV2) {
    let _ = Box::from_raw(ptr);
}

/// Returns the number of user bytes that will fit into a staged sector.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_max_user_bytes_per_staged_sector_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> u64 {
    u64::from(UnpaddedBytesAmount::from(
        RegisteredSealProof::from(registered_proof).sector_size(),
    ))
}

/// Returns the CID of the Groth parameter file for sealing.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_seal_params_cid_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> *mut fil_StringResponseV2 {
    registered_seal_proof_accessor_v2(registered_proof, RegisteredSealProof::params_cid)
}

/// Returns the CID of the verifying key-file for verifying a seal proof.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_seal_verifying_key_cid_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> *mut fil_StringResponseV2 {
    registered_seal_proof_accessor_v2(registered_proof, RegisteredSealProof::verifying_key_cid)
}

/// Returns the path from which the proofs library expects to find the Groth
/// parameter file used when sealing.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_seal_params_path_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> *mut fil_StringResponseV2 {
    registered_seal_proof_accessor_v2(registered_proof, |p| {
        p.cache_params_path()
            .map(|pb| String::from(pb.to_string_lossy()))
    })
}

/// Returns the path from which the proofs library expects to find the verifying
/// key-file used when verifying a seal proof.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_seal_verifying_key_path_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> *mut fil_StringResponseV2 {
    registered_seal_proof_accessor_v2(registered_proof, |p| {
        p.cache_verifying_key_path()
            .map(|pb| String::from(pb.to_string_lossy()))
    })
}

/// Returns the identity of the circuit for the provided seal proof.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_seal_circuit_identifier_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> *mut fil_StringResponseV2 {
    registered_seal_proof_accessor_v2(registered_proof, RegisteredSealProof::circuit_identifier)
}

/// Returns the version of the provided seal proof type.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_seal_version_v2(
    registered_proof: fil_RegisteredSealProofV2,
) -> *mut fil_StringResponseV2 {
    registered_seal_proof_accessor_v2(registered_proof, |p| Ok(format!("{:?}", p)))
}

/// Returns the CID of the Groth parameter file for generating a PoSt.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_post_params_cid_v2(
    registered_proof: fil_RegisteredPoStProofV2,
) -> *mut fil_StringResponseV2 {
    registered_post_proof_accessor_v2(registered_proof, RegisteredPoStProof::params_cid)
}

/// Returns the CID of the verifying key-file for verifying a PoSt proof.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_post_verifying_key_cid_v2(
    registered_proof: fil_RegisteredPoStProofV2,
) -> *mut fil_StringResponseV2 {
    registered_post_proof_accessor_v2(registered_proof, RegisteredPoStProof::verifying_key_cid)
}

/// Returns the path from which the proofs library expects to find the Groth
/// parameter file used when generating a PoSt.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_post_params_path_v2(
    registered_proof: fil_RegisteredPoStProofV2,
) -> *mut fil_StringResponseV2 {
    registered_post_proof_accessor_v2(registered_proof, |p| {
        p.cache_params_path()
            .map(|pb| String::from(pb.to_string_lossy()))
    })
}

/// Returns the path from which the proofs library expects to find the verifying
/// key-file used when verifying a PoSt proof.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_post_verifying_key_path_v2(
    registered_proof: fil_RegisteredPoStProofV2,
) -> *mut fil_StringResponseV2 {
    registered_post_proof_accessor_v2(registered_proof, |p| {
        p.cache_verifying_key_path()
            .map(|pb| String::from(pb.to_string_lossy()))
    })
}

/// Returns the identity of the circuit for the provided PoSt proof type.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_post_circuit_identifier_v2(
    registered_proof: fil_RegisteredPoStProofV2,
) -> *mut fil_StringResponseV2 {
    registered_post_proof_accessor_v2(registered_proof, RegisteredPoStProof::circuit_identifier)
}

/// Returns the version of the provided seal proof.
///
#[no_mangle]
pub unsafe extern "C" fn fil_get_post_version_v2(
    registered_proof: fil_RegisteredPoStProofV2,
) -> *mut fil_StringResponseV2 {
    registered_post_proof_accessor_v2(registered_proof, |p| Ok(format!("{:?}", p)))
}

unsafe fn registered_seal_proof_accessor_v2(
    registered_proof: fil_RegisteredSealProofV2,
    op: fn(RegisteredSealProof) -> anyhow::Result<String>,
) -> *mut fil_StringResponseV2 {
    let mut response = fil_StringResponseV2::default();

    let rsp: RegisteredSealProof = registered_proof.into();

    match op(rsp) {
        Ok(s) => {
            response.status_code = FCPResponseStatusV2::FCPNoError;
            response.string_val = rust_str_to_c_str(s);
        }
        Err(err) => {
            response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
            response.error_msg = rust_str_to_c_str(format!("{:?}", err));
        }
    }

    raw_ptr(response)
}

unsafe fn registered_post_proof_accessor_v2(
    registered_proof: fil_RegisteredPoStProofV2,
    op: fn(RegisteredPoStProof) -> anyhow::Result<String>,
) -> *mut fil_StringResponseV2 {
    let mut response = fil_StringResponseV2::default();

    let rsp: RegisteredPoStProof = registered_proof.into();

    match op(rsp) {
        Ok(s) => {
            response.status_code = FCPResponseStatusV2::FCPNoError;
            response.string_val = rust_str_to_c_str(s);
        }
        Err(err) => {
            response.status_code = FCPResponseStatusV2::FCPUnclassifiedError;
            response.error_msg = rust_str_to_c_str(format!("{:?}", err));
        }
    }

    raw_ptr(response)
}

/// Deallocates a VerifySealResponse.
///
#[no_mangle]
pub unsafe extern "C" fn fil_destroy_verify_seal_response_v2(ptr: *mut fil_VerifySealResponseV2) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_finalize_ticket_response_v2(
    ptr: *mut fil_FinalizeTicketResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

/// Deallocates a VerifyPoStResponse.
///
#[no_mangle]
pub unsafe extern "C" fn fil_destroy_verify_winning_post_response_v2(
    ptr: *mut fil_VerifyWinningPoStResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_verify_window_post_response_v2(
    ptr: *mut fil_VerifyWindowPoStResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_generate_winning_post_response_v2(
    ptr: *mut fil_GenerateWinningPoStResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_generate_window_post_response_v2(
    ptr: *mut fil_GenerateWindowPoStResponseV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_generate_winning_post_sector_challenge_v2(
    ptr: *mut fil_GenerateWinningPoStSectorChallengeV2,
) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn fil_destroy_clear_cache_response_v2(ptr: *mut fil_ClearCacheResponseV2) {
    let _ = Box::from_raw(ptr);
}

#[cfg(test)]
pub mod tests {
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::unix::io::IntoRawFd;

    use anyhow::Result;
    use ffi_toolkit::{c_str_to_rust_str};
    use rand::{thread_rng, Rng};

    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_write_with_and_without_alignment() -> Result<()> {
        let registered_proof = fil_RegisteredSealProofV2::StackedDrg2KiBV2;

        // write some bytes to a temp file to be used as the byte source
        let mut rng = thread_rng();
        let buf: Vec<u8> = (0..508).map(|_| rng.gen()).collect();

        // first temp file occupies 4 nodes in a merkle tree built over the
        // destination (after preprocessing)
        let mut src_file_a = tempfile::tempfile()?;
        src_file_a.write_all(&buf[0..127])?;
        src_file_a.seek(SeekFrom::Start(0))?;

        // second occupies 16 nodes
        let mut src_file_b = tempfile::tempfile()?;
        src_file_b.write_all(&buf[0..508])?;
        src_file_b.seek(SeekFrom::Start(0))?;

        // create a temp file to be used as the byte destination
        let dest = tempfile::tempfile()?;

        // transmute temp files to file descriptors
        let src_fd_a = src_file_a.into_raw_fd();
        let src_fd_b = src_file_b.into_raw_fd();
        let dst_fd = dest.into_raw_fd();

        // write the first file
        unsafe {
            let resp = fil_write_without_alignment_v2(registered_proof, src_fd_a, 127, dst_fd);

            if (*resp).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp).error_msg);
                panic!("write_without_alignment failed: {:?}", msg);
            }

            assert_eq!(
                (*resp).total_write_unpadded,
                127,
                "should have added 127 bytes of (unpadded) left alignment"
            );
        }

        // write the second
        unsafe {
            let existing = vec![127u64];

            let resp = fil_write_with_alignment_v2(
                registered_proof,
                src_fd_b,
                508,
                dst_fd,
                existing.as_ptr(),
                existing.len(),
            );

            if (*resp).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp).error_msg);
                panic!("write_with_alignment failed: {:?}", msg);
            }

            assert_eq!(
                (*resp).left_alignment_unpadded,
                381,
                "should have added 381 bytes of (unpadded) left alignment"
            );
        }

        Ok(())
    }

    #[test]
    fn test_proof_types() -> Result<()> {
        let seal_types = vec![
            fil_RegisteredSealProofV2::StackedDrg2KiBV2,
            fil_RegisteredSealProofV2::StackedDrg8MiBV2,
            fil_RegisteredSealProofV2::StackedDrg512MiBV2,
            fil_RegisteredSealProofV2::StackedDrg32GiBV2,
        ];

        let post_types = vec![
            fil_RegisteredPoStProofV2::StackedDrgWinning2KiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWinning8MiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWinning512MiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWinning32GiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWindow2KiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWindow8MiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWindow512MiBV2,
            fil_RegisteredPoStProofV2::StackedDrgWindow32GiBV2,
        ];

        let num_ops = (seal_types.len() + post_types.len()) * 6;

        let mut pairs: Vec<(&str, *mut fil_StringResponseV2)> = Vec::with_capacity(num_ops);

        unsafe {
            for st in seal_types {
                pairs.push(("get_seal_params_cid", fil_get_seal_params_cid_v2(st)));
                pairs.push((
                    "get_seal_verify_key_cid",
                    fil_get_seal_verifying_key_cid_v2(st),
                ));
                pairs.push(("get_seal_verify_key_cid", fil_get_seal_params_path_v2(st)));
                pairs.push((
                    "get_seal_verify_key_cid",
                    fil_get_seal_verifying_key_path_v2(st),
                ));
                pairs.push((
                    "get_seal_circuit_identifier",
                    fil_get_seal_circuit_identifier_v2(st),
                ));
                pairs.push(("get_seal_version", fil_get_seal_version_v2(st)));
            }

            for pt in post_types {
                pairs.push(("get_post_params_cid", fil_get_post_params_cid_v2(pt)));
                pairs.push((
                    "get_post_verify_key_cid",
                    fil_get_post_verifying_key_cid_v2(pt),
                ));
                pairs.push(("get_post_params_path", fil_get_post_params_path_v2(pt)));
                pairs.push((
                    "get_post_verifying_key_path",
                    fil_get_post_verifying_key_path_v2(pt),
                ));
                pairs.push((
                    "get_post_circuit_identifier",
                    fil_get_post_circuit_identifier_v2(pt),
                ));
                pairs.push(("get_post_version", fil_get_post_version_v2(pt)));
            }
        }

        for (label, r) in pairs {
            unsafe {
                assert_eq!(
                    (*r).status_code,
                    FCPResponseStatusV2::FCPNoError,
                    "non-success exit code from {:?}: {:?}",
                    label,
                    c_str_to_rust_str((*r).error_msg)
                );

                let x = CStr::from_ptr((*r).string_val);
                let y = x.to_str().unwrap();

                assert!(!y.is_empty());

                fil_destroy_string_response_v2(r);
            }
        }

        Ok(())
    }

    #[test]
    fn test_sealing() -> Result<()> {
        let wrap = |x| fil_32ByteArrayV2 { inner: x };

        // miscellaneous setup and shared values
        let registered_proof_seal = fil_RegisteredSealProofV2::StackedDrg2KiBV2;
        let registered_proof_winning_post = fil_RegisteredPoStProofV2::StackedDrgWinning2KiBV2;
        let registered_proof_window_post = fil_RegisteredPoStProofV2::StackedDrgWindow2KiBV2;

        let cache_dir = tempfile::tempdir()?;
        let cache_dir_path = cache_dir.into_path();

        let prover_id = fil_32ByteArrayV2 { inner: [1u8; 32] };
        let randomness = fil_32ByteArrayV2 { inner: [7u8; 32] };
        let sector_id = 42;
        let seed = fil_32ByteArrayV2 { inner: [5u8; 32] };
        let ticket = fil_32ByteArrayV2 { inner: [6u8; 32] };

        // create a byte source (a user's piece)
        let mut rng = thread_rng();
        let buf_a: Vec<u8> = (0..2032).map(|_| rng.gen()).collect();

        let mut piece_file_a = tempfile::tempfile()?;
        piece_file_a.write_all(&buf_a[0..127])?;
        piece_file_a.seek(SeekFrom::Start(0))?;

        let mut piece_file_b = tempfile::tempfile()?;
        piece_file_b.write_all(&buf_a[0..1016])?;
        piece_file_b.seek(SeekFrom::Start(0))?;

        // create the staged sector (the byte destination)
        let (staged_file, staged_path) = tempfile::NamedTempFile::new()?.keep()?;

        // create a temp file to be used as the byte destination
        let (sealed_file, sealed_path) = tempfile::NamedTempFile::new()?.keep()?;

        // last temp file is used to output unsealed bytes
        let (unseal_file, unseal_path) = tempfile::NamedTempFile::new()?.keep()?;

        // transmute temp files to file descriptors
        let piece_file_a_fd = piece_file_a.into_raw_fd();
        let piece_file_b_fd = piece_file_b.into_raw_fd();
        let staged_sector_fd = staged_file.into_raw_fd();

        unsafe {
            let resp_a1 = fil_write_without_alignment_v2(
                registered_proof_seal,
                piece_file_a_fd,
                127,
                staged_sector_fd,
            );

            if (*resp_a1).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_a1).error_msg);
                panic!("write_without_alignment failed: {:?}", msg);
            }

            let existing_piece_sizes = vec![127];

            let resp_a2 = fil_write_with_alignment_v2(
                registered_proof_seal,
                piece_file_b_fd,
                1016,
                staged_sector_fd,
                existing_piece_sizes.as_ptr(),
                existing_piece_sizes.len(),
            );

            if (*resp_a2).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_a2).error_msg);
                panic!("write_with_alignment failed: {:?}", msg);
            }

            let pieces = vec![
                fil_PublicPieceInfoV2 {
                    num_bytes: 127,
                    comm_p: (*resp_a1).comm_p,
                },
                fil_PublicPieceInfoV2 {
                    num_bytes: 1016,
                    comm_p: (*resp_a2).comm_p,
                },
            ];

            let resp_x =
                fil_generate_data_commitment_v2(registered_proof_seal, pieces.as_ptr(), pieces.len());

            if (*resp_x).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_x).error_msg);
                panic!("generate_data_commitment failed: {:?}", msg);
            }

            let cache_dir_path_c_str = rust_str_to_c_str(cache_dir_path.to_str().unwrap());
            let staged_path_c_str = rust_str_to_c_str(staged_path.to_str().unwrap());
            let replica_path_c_str = rust_str_to_c_str(sealed_path.to_str().unwrap());
            let unseal_path_c_str = rust_str_to_c_str(unseal_path.to_str().unwrap());

            let resp_b1 = fil_seal_pre_commit_phase1_v2(
                registered_proof_seal,
                cache_dir_path_c_str,
                staged_path_c_str,
                replica_path_c_str,
                sector_id,
                prover_id,
                ticket,
                pieces.as_ptr(),
                pieces.len(),
            );

            if (*resp_b1).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_b1).error_msg);
                panic!("seal_pre_commit_phase1 failed: {:?}", msg);
            }

            let resp_b2 = fil_seal_pre_commit_phase2_v2(
                (*resp_b1).seal_pre_commit_phase1_output_ptr,
                (*resp_b1).seal_pre_commit_phase1_output_len,
                cache_dir_path_c_str,
                replica_path_c_str,
            );

            if (*resp_b2).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_b2).error_msg);
                panic!("seal_pre_commit_phase2 failed: {:?}", msg);
            }

            let pre_computed_comm_d = &(*resp_x).comm_d;
            let pre_commit_comm_d = &(*resp_b2).comm_d;

            assert_eq!(
                format!("{:x?}", &pre_computed_comm_d),
                format!("{:x?}", &pre_commit_comm_d),
                "pre-computed CommD and pre-commit CommD don't match"
            );

            let resp_c1 = fil_seal_commit_phase1_v2(
                registered_proof_seal,
                wrap((*resp_b2).comm_r),
                wrap((*resp_b2).comm_d),
                cache_dir_path_c_str,
                replica_path_c_str,
                sector_id,
                prover_id,
                ticket,
                seed,
                pieces.as_ptr(),
                pieces.len(),
            );

            if (*resp_c1).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_c1).error_msg);
                panic!("seal_commit_phase1 failed: {:?}", msg);
            }

            let resp_c2 = fil_seal_commit_phase2_v2(
                (*resp_c1).seal_commit_phase1_output_ptr,
                (*resp_c1).seal_commit_phase1_output_len,
                sector_id,
                prover_id,
            );

            if (*resp_c2).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_c2).error_msg);
                panic!("seal_commit_phase2 failed: {:?}", msg);
            }

            let resp_d = fil_verify_seal_v2(
                registered_proof_seal,
                wrap((*resp_b2).comm_r),
                wrap((*resp_b2).comm_d),
                prover_id,
                ticket,
                seed,
                sector_id,
                (*resp_c2).proof_ptr,
                (*resp_c2).proof_len,
            );

            if (*resp_d).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_d).error_msg);
                panic!("seal_commit failed: {:?}", msg);
            }

            assert!((*resp_d).is_valid, "proof was not valid");

            let resp_e = fil_unseal_range_v2(
                registered_proof_seal,
                cache_dir_path_c_str,
                sealed_file.into_raw_fd(),
                unseal_file.into_raw_fd(),
                sector_id,
                prover_id,
                ticket,
                wrap((*resp_b2).comm_d),
                0,
                2032,
            );

            if (*resp_e).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_e).error_msg);
                panic!("unseal failed: {:?}", msg);
            }

            // ensure unsealed bytes match what we had in our piece
            let mut buf_b = Vec::with_capacity(2032);
            let mut f = std::fs::File::open(unseal_path)?;

            let _ = f.read_to_end(&mut buf_b)?;

            let piece_a_len = (*resp_a1).total_write_unpadded as usize;
            let piece_b_len = (*resp_a2).total_write_unpadded as usize;
            let piece_b_prefix_len = (*resp_a2).left_alignment_unpadded as usize;

            let alignment = vec![0; piece_b_prefix_len];

            let expected = [
                &buf_a[0..piece_a_len],
                &alignment[..],
                &buf_a[0..(piece_b_len - piece_b_prefix_len)],
            ]
            .concat();

            assert_eq!(
                format!("{:x?}", &expected),
                format!("{:x?}", &buf_b),
                "original bytes don't match unsealed bytes"
            );

            // generate a PoSt

            let sectors = vec![sector_id];
            let resp_f = fil_generate_winning_post_sector_challenge_v2(
                registered_proof_winning_post,
                randomness,
                sectors.len() as u64,
                prover_id,
            );

            if (*resp_f).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_f).error_msg);
                panic!("generate_candidates failed: {:?}", msg);
            }

            // exercise the ticket-finalizing code path (but don't do anything
            // with the results
            let result: &[u64] = from_raw_parts((*resp_f).ids_ptr, (*resp_f).ids_len);

            if result.is_empty() {
                panic!("generate_candidates produced no results");
            }

            let private_replicas = vec![fil_PrivateReplicaInfoV2 {
                registered_proof: registered_proof_winning_post,
                cache_dir_path: cache_dir_path_c_str,
                comm_r: (*resp_b2).comm_r,
                replica_path: replica_path_c_str,
                sector_id,
            }];

            // winning post

            let resp_h = fil_generate_winning_post_v2(
                randomness,
                private_replicas.as_ptr(),
                private_replicas.len(),
                prover_id,
            );

            if (*resp_h).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_h).error_msg);
                panic!("generate_winning_post failed: {:?}", msg);
            }
            let public_replicas = vec![fil_PublicReplicaInfoV2 {
                registered_proof: registered_proof_winning_post,
                sector_id,
                comm_r: (*resp_b2).comm_r,
            }];

            let resp_i = fil_verify_winning_post_v2(
                randomness,
                public_replicas.as_ptr(),
                public_replicas.len(),
                (*resp_h).proofs_ptr,
                (*resp_h).proofs_len,
                prover_id,
            );

            if (*resp_i).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_i).error_msg);
                panic!("verify_winning_post failed: {:?}", msg);
            }

            if !(*resp_i).is_valid {
                panic!("verify_winning_post rejected the provided proof as invalid");
            }

            // window post

            let private_replicas = vec![fil_PrivateReplicaInfoV2 {
                registered_proof: registered_proof_window_post,
                cache_dir_path: cache_dir_path_c_str,
                comm_r: (*resp_b2).comm_r,
                replica_path: replica_path_c_str,
                sector_id,
            }];

            let resp_j = fil_generate_window_post_v2(
                randomness,
                private_replicas.as_ptr(),
                private_replicas.len(),
                prover_id,
            );

            if (*resp_j).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_j).error_msg);
                panic!("generate_window_post failed: {:?}", msg);
            }

            let public_replicas = vec![fil_PublicReplicaInfoV2 {
                registered_proof: registered_proof_window_post,
                sector_id,
                comm_r: (*resp_b2).comm_r,
            }];

            let resp_k = fil_verify_window_post_v2(
                randomness,
                public_replicas.as_ptr(),
                public_replicas.len(),
                (*resp_j).proofs_ptr,
                (*resp_j).proofs_len,
                prover_id,
            );

            if (*resp_k).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_k).error_msg);
                panic!("verify_window_post failed: {:?}", msg);
            }

            if !(*resp_k).is_valid {
                panic!("verify_window_post rejected the provided proof as invalid");
            }

            fil_destroy_write_without_alignment_response_v2(resp_a1);
            fil_destroy_write_with_alignment_response_v2(resp_a2);
            fil_destroy_generate_data_commitment_response_v2(resp_x);

            fil_destroy_seal_pre_commit_phase1_response_v2(resp_b1);
            fil_destroy_seal_pre_commit_phase2_response_v2(resp_b2);
            fil_destroy_seal_commit_phase1_response_v2(resp_c1);
            fil_destroy_seal_commit_phase2_response_v2(resp_c2);

            fil_destroy_verify_seal_response_v2(resp_d);
            fil_destroy_unseal_range_response_v2(resp_e);

            fil_destroy_generate_winning_post_sector_challenge_v2(resp_f);
            fil_destroy_generate_winning_post_response_v2(resp_h);
            fil_destroy_verify_winning_post_response_v2(resp_i);

            fil_destroy_generate_window_post_response_v2(resp_j);
            fil_destroy_verify_window_post_response_v2(resp_k);

            c_str_to_rust_str(cache_dir_path_c_str);
            c_str_to_rust_str(staged_path_c_str);
            c_str_to_rust_str(replica_path_c_str);
            c_str_to_rust_str(unseal_path_c_str);
        }

        Ok(())
    }

    #[test]
    fn test_faulty_sectors() -> Result<()> {
        // miscellaneous setup and shared values
        let registered_proof_seal = fil_RegisteredSealProofV2::StackedDrg2KiBV2;
        let registered_proof_window_post = fil_RegisteredPoStProofV2::StackedDrgWindow2KiBV2;

        let cache_dir = tempfile::tempdir()?;
        let cache_dir_path = cache_dir.into_path();

        let prover_id = fil_32ByteArrayV2 { inner: [1u8; 32] };
        let randomness = fil_32ByteArrayV2 { inner: [7u8; 32] };
        let sector_id = 42;
        let ticket = fil_32ByteArrayV2 { inner: [6u8; 32] };

        // create a byte source (a user's piece)
        let mut rng = thread_rng();
        let buf_a: Vec<u8> = (0..2032).map(|_| rng.gen()).collect();

        let mut piece_file_a = tempfile::tempfile()?;
        piece_file_a.write_all(&buf_a[0..127])?;
        piece_file_a.seek(SeekFrom::Start(0))?;

        let mut piece_file_b = tempfile::tempfile()?;
        piece_file_b.write_all(&buf_a[0..1016])?;
        piece_file_b.seek(SeekFrom::Start(0))?;

        // create the staged sector (the byte destination)
        let (staged_file, staged_path) = tempfile::NamedTempFile::new()?.keep()?;

        // create a temp file to be used as the byte destination
        let (_sealed_file, sealed_path) = tempfile::NamedTempFile::new()?.keep()?;

        // transmute temp files to file descriptors
        let piece_file_a_fd = piece_file_a.into_raw_fd();
        let piece_file_b_fd = piece_file_b.into_raw_fd();
        let staged_sector_fd = staged_file.into_raw_fd();

        unsafe {
            let resp_a1 = fil_write_without_alignment_v2(
                registered_proof_seal,
                piece_file_a_fd,
                127,
                staged_sector_fd,
            );

            if (*resp_a1).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_a1).error_msg);
                panic!("write_without_alignment failed: {:?}", msg);
            }

            let existing_piece_sizes = vec![127];

            let resp_a2 = fil_write_with_alignment_v2(
                registered_proof_seal,
                piece_file_b_fd,
                1016,
                staged_sector_fd,
                existing_piece_sizes.as_ptr(),
                existing_piece_sizes.len(),
            );

            if (*resp_a2).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_a2).error_msg);
                panic!("write_with_alignment failed: {:?}", msg);
            }

            let pieces = vec![
                fil_PublicPieceInfoV2 {
                    num_bytes: 127,
                    comm_p: (*resp_a1).comm_p,
                },
                fil_PublicPieceInfoV2 {
                    num_bytes: 1016,
                    comm_p: (*resp_a2).comm_p,
                },
            ];

            let resp_x =
                fil_generate_data_commitment_v2(registered_proof_seal, pieces.as_ptr(), pieces.len());

            if (*resp_x).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_x).error_msg);
                panic!("generate_data_commitment failed: {:?}", msg);
            }

            let cache_dir_path_c_str = rust_str_to_c_str(cache_dir_path.to_str().unwrap());
            let staged_path_c_str = rust_str_to_c_str(staged_path.to_str().unwrap());
            let replica_path_c_str = rust_str_to_c_str(sealed_path.to_str().unwrap());

            let resp_b1 = fil_seal_pre_commit_phase1_v2(
                registered_proof_seal,
                cache_dir_path_c_str,
                staged_path_c_str,
                replica_path_c_str,
                sector_id,
                prover_id,
                ticket,
                pieces.as_ptr(),
                pieces.len(),
            );

            if (*resp_b1).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_b1).error_msg);
                panic!("seal_pre_commit_phase1 failed: {:?}", msg);
            }

            let resp_b2 = fil_seal_pre_commit_phase2_v2(
                (*resp_b1).seal_pre_commit_phase1_output_ptr,
                (*resp_b1).seal_pre_commit_phase1_output_len,
                cache_dir_path_c_str,
                replica_path_c_str,
            );

            if (*resp_b2).status_code != FCPResponseStatusV2::FCPNoError {
                let msg = c_str_to_rust_str((*resp_b2).error_msg);
                panic!("seal_pre_commit_phase2 failed: {:?}", msg);
            }

            // window post

            let faulty_sealed_file = tempfile::NamedTempFile::new()?;
            let faulty_replica_path_c_str =
                rust_str_to_c_str(faulty_sealed_file.path().to_str().unwrap());

            let private_replicas = vec![fil_PrivateReplicaInfoV2 {
                registered_proof: registered_proof_window_post,
                cache_dir_path: cache_dir_path_c_str,
                comm_r: (*resp_b2).comm_r,
                replica_path: faulty_replica_path_c_str,
                sector_id,
            }];

            let resp_j = fil_generate_window_post_v2(
                randomness,
                private_replicas.as_ptr(),
                private_replicas.len(),
                prover_id,
            );

            assert_eq!(
                (*resp_j).status_code,
                FCPResponseStatusV2::FCPUnclassifiedError,
                "generate_window_post should have failed"
            );

            let faulty_sectors: &[u64] =
                from_raw_parts((*resp_j).faulty_sectors_ptr, (*resp_j).faulty_sectors_len);
            assert_eq!(faulty_sectors, &[42], "sector 42 should be faulty");

            fil_destroy_write_without_alignment_response_v2(resp_a1);
            fil_destroy_write_with_alignment_response_v2(resp_a2);

            fil_destroy_seal_pre_commit_phase1_response_v2(resp_b1);
            fil_destroy_seal_pre_commit_phase2_response_v2(resp_b2);

            fil_destroy_generate_window_post_response_v2(resp_j);

            c_str_to_rust_str(cache_dir_path_c_str);
            c_str_to_rust_str(staged_path_c_str);
            c_str_to_rust_str(replica_path_c_str);
        }

        Ok(())
    }
}
