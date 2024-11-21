(define (domain bs-domain)
 (:requirements :strips :typing :numeric-fluents :durative-actions)
 (:types application agent mobile list message network)
 (:constants
   ae - agent
 )
 (:predicates (initiated ?a_new - application ?m - mobile) (qos_params ?a_new - application ?l - list) (trm_ok ?a_new - application ?m - mobile ?l - list) (ct_ok ?a_new - application ?m - mobile ?l - list) (location ?m - mobile) (authentification ?m - mobile) (am_ok ?a_new - application ?m - mobile ?l - list) (aeem_ok ?a_new - application ?m - mobile ?l - list ?a - agent) (rrc_ok ?a_new - application ?m - mobile ?l - list ?a - agent) (rab_ok ?a_new - application ?m - mobile ?l - list ?a - agent) (aeei_ok ?a_new - application ?m - mobile ?l - list ?a - agent) (message_trm ?m - mobile ?ms - message) (message_aeei ?a_new - application ?ms - message) (iu_bearer ?a_new - application ?m - mobile ?l - list) (bs_ok ?a_new - application ?m - mobile ?l - list ?a - agent) (begin_init ?a - agent) (begin_aeei ?a - agent) (m) (n) (p0) (p1) (p2) (p3) (p4) (pn))
 (:functions (app_cpu ?a_new - application ?m - mobile) (app_display ?a_new - application ?m - mobile) (app_keyboard ?a_new - application ?m - mobile) (app_energy ?a_new - application ?m - mobile) (app_channels ?a_new - application ?m - mobile) (list_pdu ?l - list) (list_ml ?l - list) (time_trm ?a_new - application) (time_ct ?a_new - application) (time_am ?a_new - application) (time_aeem ?a_new - application) (time_rrc ?a_new - application) (time_rrc_negotiation ?a_new - application) (time_rab ?a_new - application) (time_aeei ?a_new - application) (time_bs ?a_new - application) (max_mobile_cpu) (max_d_available) (max_k_available) (max_e_load) (max_mobile_channels_available) (max_num_mobiles) (max_num_calls) (max_mobile_storage) (max_logical_channels) (max_cell_update) (max_handover) (max_active_set_up) (max_ggsn_bitrate) (max_no_pdp) (max_no_apn) (has_mobile_cpu) (has_d_available) (has_k_available) (has_e_load) (has_mobile_channels_available) (has_num_mobiles) (has_num_calls) (has_mobile_storage) (has_logical_channels) (has_cell_update) (has_handover) (has_active_set_up) (has_ggsn_bitrate) (has_max_no_pdp) (has_max_no_apn))
 (:durative-action trm
  :parameters ( ?a_new - application ?m - mobile ?l - list)
  :duration (= ?duration (time_trm ?a_new))
  :condition (and (at start (n))(at start (initiated ?a_new ?m))(at start (qos_params ?a_new ?l))(at start (<= (has_mobile_cpu) (- (max_mobile_cpu) (app_cpu ?a_new ?m))))(at start (<= (has_d_available) (- (max_d_available) (app_display ?a_new ?m))))(at start (<= (has_k_available) (- (max_k_available) (app_keyboard ?a_new ?m))))(at start (<= (has_e_load) (- (max_e_load) (app_energy ?a_new ?m))))(at start (<= (has_mobile_channels_available) (- (max_mobile_channels_available) (app_channels ?a_new ?m)))))
  :effect (and (at end (trm_ok ?a_new ?m ?l)) (at end (decrease (has_mobile_cpu) (app_cpu ?a_new ?m))) (at end (decrease (has_d_available) (app_display ?a_new ?m))) (at end (decrease (has_k_available) (app_keyboard ?a_new ?m))) (at end (decrease (has_e_load) (app_energy ?a_new ?m))) (at end (decrease (has_mobile_channels_available) (app_channels ?a_new ?m))) (at start (increase (has_mobile_cpu) (app_cpu ?a_new ?m))) (at start (increase (has_d_available) (app_display ?a_new ?m))) (at start (increase (has_k_available) (app_keyboard ?a_new ?m))) (at start (increase (has_e_load) (app_energy ?a_new ?m))) (at start (increase (has_mobile_channels_available) (app_channels ?a_new ?m)))))
 (:durative-action ct
  :parameters ( ?a_new - application ?m - mobile ?l - list)
  :duration (= ?duration (time_ct ?a_new))
  :condition (and (at start (n))(at start (trm_ok ?a_new ?m ?l))(at start (qos_params ?a_new ?l))(at start (< (has_num_mobiles) (max_num_mobiles)))(at start (< (has_num_calls) (max_num_calls))))
  :effect (and (at start (increase (has_num_mobiles) 1)) (at start (increase (has_num_calls) 1)) (at end (decrease (has_num_mobiles) 1)) (at end (decrease (has_num_calls) 1)) (at end (ct_ok ?a_new ?m ?l))))
 (:durative-action am
  :parameters ( ?a_new - application ?m - mobile ?l - list)
  :duration (= ?duration 0)
  :condition (and (at start (n))(at start (ct_ok ?a_new ?m ?l))(at start (location ?m))(at start (authentification ?m)))
  :effect (and (at end (am_ok ?a_new ?m ?l))))
 (:durative-action aeem
  :parameters ( ?a_new - application ?m - mobile ?l - list ?a - agent)
  :duration (= ?duration (time_aeem ?a_new))
  :condition (and (at start (n))(at start (am_ok ?a_new ?m ?l))(at start (<= (has_mobile_storage) (- (max_mobile_storage) 10)))(at start (begin_init ?a)))
  :effect (and (at start (increase (has_mobile_storage) 10)) (at end (decrease (has_mobile_storage) 10)) (at end (aeem_ok ?a_new ?m ?l ?a))))
 (:durative-action rrc
  :parameters ( ?a_new - application ?m - mobile ?l - list ?a - agent)
  :duration (= ?duration (time_rrc ?a_new))
  :condition (and (at start (n))(at start (ct_ok ?a_new ?m ?l))(at start (aeem_ok ?a_new ?m ?l ?a))(at start (<= (has_logical_channels) (- (max_logical_channels) (app_channels ?a_new ?m))))(at start (<= (has_cell_update) (- (max_cell_update) 2)))(at start (< (has_handover) (max_handover)))(at start (< (has_active_set_up) (max_active_set_up))))
  :effect (and (at start (increase (has_logical_channels) (app_channels ?a_new ?m))) (at start (increase (has_cell_update) 2)) (at start (increase (has_handover) 1)) (at start (increase (has_active_set_up) 1)) (at end (decrease (has_logical_channels) (app_channels ?a_new ?m))) (at end (decrease (has_cell_update) 2)) (at end (decrease (has_handover) 1)) (at end (decrease (has_active_set_up) 1)) (at end (rrc_ok ?a_new ?m ?l ?a))))
 (:durative-action rab
  :parameters ( ?a_new - application ?m - mobile ?l - list ?a - agent)
  :duration (= ?duration (time_rab ?a_new))
  :condition (and (at start (n))(at start (rrc_ok ?a_new ?m ?l ?a)))
  :effect (and (at end (rab_ok ?a_new ?m ?l ?a))))
 (:durative-action aeei
  :parameters ( ?a_new - application ?m - mobile ?l - list ?a - agent)
  :duration (= ?duration (time_aeei ?a_new))
  :condition (and (at start (n))(at start (rab_ok ?a_new ?m ?l ?a))(at start (<= (has_ggsn_bitrate) (- (max_ggsn_bitrate) 128)))(at start (< (has_max_no_pdp) (max_no_pdp)))(at start (< (has_max_no_apn) (max_no_apn)))(at start (begin_aeei ?a)))
  :effect (and (at end (aeei_ok ?a_new ?m ?l ?a)) (at end (decrease (has_ggsn_bitrate) 128)) (at end (decrease (has_max_no_pdp) 1)) (at end (decrease (has_max_no_apn) 1)) (at start (increase (has_ggsn_bitrate) 128)) (at start (increase (has_max_no_pdp) 1)) (at start (increase (has_max_no_apn) 1))))
 (:durative-action bs
  :parameters ( ?a_new - application ?m - mobile ?l - list ?ms1 - message ?ms2 - message ?a - agent)
  :duration (= ?duration (time_bs ?a_new))
  :condition (and (at start (n))(at start (initiated ?a_new ?m))(at start (aeei_ok ?a_new ?m ?l ?a))(at start (qos_params ?a_new ?l))(at start (message_trm ?m ?ms1))(at start (message_aeei ?a_new ?ms2)))
  :effect (and (at end (iu_bearer ?a_new ?m ?l)) (at end (bs_ok ?a_new ?m ?l ?a))))
 (:durative-action timedliteralwrapper
  :parameters ()
  :duration (= ?duration 2151)
  :condition (and (at start (m)))
  :effect (and (at start (not (m))) (at start (n)) (at start (p0)) (at start (pn)) (at end (not (pn)))))
 (:durative-action timedliteral1
  :parameters ()
  :duration (= ?duration 70)
  :condition (and (over all (p0))(over all (pn)))
  :effect (and (at end (p1)) (at end (not (p0))) (at end (begin_init ae))))
 (:durative-action timedliteral2
  :parameters ()
  :duration (= ?duration 691)
  :condition (and (over all (p1))(over all (pn)))
  :effect (and (at end (p2)) (at end (not (p1))) (at end (not (begin_init ae)))))
 (:durative-action timedliteral3
  :parameters ()
  :duration (= ?duration 669)
  :condition (and (over all (p2))(over all (pn)))
  :effect (and (at end (p3)) (at end (not (p2))) (at end (begin_aeei ae))))
 (:durative-action timedliteral4
  :parameters ()
  :duration (= ?duration 721)
  :condition (and (over all (p3)))
  :effect (and (at end (p4)) (at end (not (p3))) (at end (not (begin_aeei ae)))))
)